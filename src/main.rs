mod circular_buffer;
mod hann_window;

use circular_buffer::CircularBuffer;
use hound::{WavReader, WavSpec, WavWriter};
use libm::{atan2f, cosf, floorf, fmodf, sinf, sqrtf};
use std::error::Error;
use std::sync::{Arc, Mutex};
use std::thread;
const PI: f32 = 3.14159265358979323846264338327950288f32;
const BUFFER_SIZE: usize = 100000;
const FFT_SIZE: usize = 1024;
const HOP_SIZE: usize = 128;
const PITCH_SHIFT: f32 = 1.5;

fn main() -> Result<(), Box<dyn Error>> {
    let path = "sample_f32.wav";
    // let path = "WeChooseToGoToTheMoon_f32.wav";
    let mut reader = WavReader::open(path)?;
    let spec = reader.spec();

    match spec.sample_format {
        hound::SampleFormat::Int => match spec.bits_per_sample {
            _ => return Err(Box::from("Unsupported bit depth")),
        },
        hound::SampleFormat::Float => match spec.bits_per_sample {
            32 => read_and_write_samples::<f32>(&mut reader, &spec)?,
            _ => return Err(Box::from("Unsupported bit depth")),
        },
    }

    Ok(())
}

fn read_and_write_samples<S>(
    reader: &mut WavReader<std::io::BufReader<std::fs::File>>,
    spec: &WavSpec,
) -> Result<(), Box<dyn Error>>
where
    S: hound::Sample + std::fmt::Debug + hound::Sample,
{
    let output_spec = WavSpec { ..*spec };

    let output_path = "processed_sample.wav";
    let mut writer = WavWriter::create(output_path, output_spec)?;
    let mut hop_counter = 0;

    // Wrap shared resources with Arc<Mutex<T>>
    let buffer_in = Arc::new(Mutex::new([0.0; BUFFER_SIZE]));
    let buffer_out = Arc::new(Mutex::new([0.0; BUFFER_SIZE]));

    let in_buffer_pointer = Arc::new(Mutex::new(0));
    let cached_in_buffer_pointer = Arc::new(Mutex::new(0));
    // Start the write pointer ahead of the read pointer by at least window + hop, with some margin
    let out_buffer_write_pointer = Arc::new(Mutex::new(BUFFER_SIZE + 2 * HOP_SIZE));
    let out_buffer_reader_pointer = Arc::new(Mutex::new(0));

    for sample in reader.samples::<f32>() {
        let sample = sample.expect("Error reading sample");

        // Store the sample ("in") in a buffer for the FFT
        // Increment the pointer and when full window has been
        // assembled, call process_fft()
        // NOTE FOR ENOCH - This is done in {} because rust will automatically unlock these resources when the program leaves the scope.
        {
            let mut buffer_in = buffer_in.lock().unwrap();
            let mut in_buff_ptr = in_buffer_pointer.lock().unwrap();
            buffer_in[*in_buff_ptr] = sample;
            *in_buff_ptr += 1;
            if *in_buff_ptr >= BUFFER_SIZE {
                // Wrap the circular buffer
                // Notice: this is not the condition for starting a new FFT
                *in_buff_ptr = 0;
            }
        }

        // Get the output sample from the output buffer
        let scaled_out_sample = {
            let mut buffer_out = buffer_out.lock().unwrap();
            let mut out_read_ptr = out_buffer_reader_pointer.lock().unwrap();
            let out = buffer_out[*out_read_ptr];

            // Then clear the output sample in the buffer so it is ready for the next overlap-add
            buffer_out[*out_read_ptr] = 0.0;

            // Increment the read pointer in the output cicular buffer
            *out_read_ptr += 1;
            if *out_read_ptr >= BUFFER_SIZE {
                *out_read_ptr = 0;
            }

            // Scale the output down by the scale factor, compensating for the overlap
            out * HOP_SIZE as f32 / FFT_SIZE as f32
        };

        // Increment the hop counter and start a new FFT if we've reached the hop size
        if hop_counter >= HOP_SIZE {
            hop_counter = 0;

            // Clone the Arc<Mutex<>> for the new thread
            let buffer_in_clone = Arc::clone(&buffer_in);
            let buffer_out_clone = Arc::clone(&buffer_out);
            let out_write_ptr = Arc::clone(&out_buffer_write_pointer);
            let input_ptr = {
                let in_buff_ptr = in_buffer_pointer.lock().unwrap();
                *in_buff_ptr
            };
            let output_ptr = {
                let out_buff_ptr = out_write_ptr.lock().unwrap().clone();
                out_buff_ptr
            };

            // Spawn a new thread for processing FFT
            thread::spawn(move || {
                let mut last_input_phases = [0.0; FFT_SIZE];
                let mut last_output_phases = [0.0; FFT_SIZE];
                let mut bin_frequencies = [0.0; FFT_SIZE / 2];

                {
                    let mut in_buf = buffer_in_clone.lock().unwrap();
                    let mut out_buf = buffer_out_clone.lock().unwrap();

                    process_fft(
                        &mut in_buf,
                        &mut out_buf,
                        &mut last_input_phases,
                        &mut last_output_phases,
                        &mut bin_frequencies,
                        input_ptr,
                        output_ptr
                    );
                    	// Update the output buffer write pointer to start at the next hop
                    {
                        let mut out_buff_ptr = out_write_ptr.lock().unwrap();
                        *out_buff_ptr = (*out_buff_ptr + HOP_SIZE) % BUFFER_SIZE
                    }
                    // out_buf.next_hop();
                }
            });
        }
        hop_counter += 1;
        writer.write_sample(scaled_out_sample)?;
    }

    writer.finalize()?;
    Ok(())
}

fn process_fft(
    in_buffer: &mut [f32; BUFFER_SIZE],
    out_buffer: &mut [f32; BUFFER_SIZE],
    last_input_phases: &mut [f32; FFT_SIZE],
    last_output_phases: &mut [f32; FFT_SIZE],
    bin_frequencies: &mut [f32; FFT_SIZE / 2],
    input_pointer: usize,
    output_pointer: usize,
) {
    let analysis_window_buffer: [f32; FFT_SIZE] = hann_window::HANN_WINDOW;

    let mut unwrapped_buffer: [f32; FFT_SIZE] = [0.0; FFT_SIZE];
    let mut full_spectrum: [microfft::Complex32; FFT_SIZE] =
        [microfft::Complex32 { re: 0.0, im: 0.0 }; FFT_SIZE];
    let mut analysis_magnitudes = [0.0; FFT_SIZE / 2];
    let mut analysis_frequencies = [0.0; FFT_SIZE / 2];
    let mut synthesis_magnitudes = [0.0; FFT_SIZE / 2];
    let mut synthesis_frequencies = [0.0; FFT_SIZE / 2];
    let mut synthesis_count = [0; FFT_SIZE / 2];

    // Copy buffer into FFT input, starting one window ago

    for n in 0..FFT_SIZE {
        let circular_buffer_index = (input_pointer + n - FFT_SIZE + BUFFER_SIZE) % BUFFER_SIZE;
        unwrapped_buffer[n] = in_buffer[circular_buffer_index] * analysis_window_buffer[n];
    }

    // Process the FFT based on the time domain input
    let fft = microfft::real::rfft_1024(&mut unwrapped_buffer);

    // ANALYSIS
    for i in 0..fft.len() {
        // Turn real and imaginary components into amplitude and phase
        let amplitude = sqrtf(fft[i].re * fft[i].re + fft[i].im * fft[i].im);
        let phase = atan2f(fft[i].im, fft[i].re);

        // Calculate the phase difference in this bin between the last
        // hop and this one, which will indirectly give us the exact frequency
        let mut phase_diff = phase - last_input_phases[i];

        // Subtract the amount of phase increment we'd expect to see based
        // on the centre frequency of this bin (2*pi*n/gFftSize) for this
        // hop size, then wrap to the range -pi to pi
        let bin_centre_frequency = 2.0 * PI * i as f32 / FFT_SIZE as f32;
        phase_diff = wrap_phase(phase_diff - bin_centre_frequency * HOP_SIZE as f32);

        // Find deviation from the centre frequency
        let bin_deviation = phase_diff * FFT_SIZE as f32 / HOP_SIZE as f32 / (2.0 * PI);

        // Add the original bin number to get the fractional bin where this partial belongs
        analysis_frequencies[i] = i as f32 + bin_deviation;
        // Save the magnitude for later
        analysis_magnitudes[i] = amplitude;
        // Save the phase for next hop
        last_input_phases[i] = phase;
    }

    // Zero out the synthesis bins, ready for new data (NOT done since it should already be zero)

    // Handle the pitch shift, storing frequencies into new bins
    for i in 0..FFT_SIZE / 2 {
        // Find the nearest bin to the shifted frequency
        let new_bin = floorf(i as f32 * PITCH_SHIFT + 0.5) as usize;

        // Ignore any bins that have shifted above Nyquist
        if new_bin < FFT_SIZE / 2 {
            synthesis_magnitudes[new_bin] += analysis_magnitudes[i];
            synthesis_frequencies[new_bin] = analysis_frequencies[i] * PITCH_SHIFT;
        }
    }

    // SYNTHESIS
    for i in 0..FFT_SIZE / 2 {
        let amplitude = synthesis_magnitudes[i];
        // Get the fractional offset from the bin centre frequency

        let bin_deviation = synthesis_frequencies[i] - i as f32;
        // Multiply to get back to a phase value
        let mut phase_diff = bin_deviation * 2.0 * PI * HOP_SIZE as f32 / FFT_SIZE as f32;
        // Add the expected phase increment based on the bin centre frequency
        let bin_centre_frequency = 2.0 * PI * i as f32 / FFT_SIZE as f32;
        phase_diff += bin_centre_frequency * HOP_SIZE as f32;
        // Advance the phase from the previous hop
        let out_phase = wrap_phase(last_output_phases[i] + phase_diff);

        // Now convert magnitude and phase back to real and imaginary components
        fft[i].re = amplitude * cosf(out_phase);
        fft[i].im = amplitude * sinf(out_phase);
        // Also store the complex conjugate in the upper half of the spectrum

        // Save the phase for the next hop
        last_output_phases[i] = out_phase;
    }

    // Reconstruct the full spectrum for the IFFT
    for i in 0..(FFT_SIZE / 2) {
        full_spectrum[i] = fft[i]; // First half directly
        if i > 0 && i < (FFT_SIZE / 2) {
            full_spectrum[FFT_SIZE - i] = fft[i].conj(); // Conjugate symmetry for the second half
        }
    }

    // Run the inverse FFT
    let res = microfft::inverse::ifft_1024(&mut full_spectrum);

    // Add time domain into the output buffer
    for (n, val) in res.iter().enumerate() {
        let circular_buffer_index = (output_pointer + n - FFT_SIZE + BUFFER_SIZE) % BUFFER_SIZE;
        let windowed_val = val.re * analysis_window_buffer[n]; // Window again and scale
        out_buffer[circular_buffer_index] += windowed_val;
    }
}

fn wrap_phase(phase_in: f32) -> f32 {
    if phase_in >= 0.0 {
        return fmodf(phase_in + PI, 2.0 * PI) - PI;
    }
    fmodf(phase_in - PI, -2.0 * PI) + PI
}
