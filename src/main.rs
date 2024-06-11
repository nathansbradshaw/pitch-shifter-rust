mod circular_buffer;
mod hann_window;

use circular_buffer::CircularBuffer;

use hound::{WavReader, WavSpec, WavWriter};
use libm::{powf, sqrtf};
use std::error::Error;

const BUFFER_SIZE: usize = 3000;
const FFT_SIZE: usize = 1024;
const HOP_SIZE: usize = 256;
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
    let mut buffer_in: CircularBuffer<f32, BUFFER_SIZE> = CircularBuffer::new(0.0, Some(0));
    let mut hop_counter = 0;
    let mut buffer_out: CircularBuffer<f32, BUFFER_SIZE> = CircularBuffer::new(0.0, Some(HOP_SIZE));

    for sample in reader.samples::<f32>() {
        let sample = sample.expect("Error reading sample");
        // println!("Sample: {:?}", sample);

        // Store the sample in the input buffer
        buffer_in.write(sample);

        // Read from the output buffer and reset the value
        let out_sample = buffer_out.read_and_reset();

        // Scale the output dow by the overlap factor
        let scaled_out_sample = out_sample * HOP_SIZE as f32 / FFT_SIZE as f32;

        // Increment the hop counter
        if hop_counter >= HOP_SIZE {
            hop_counter = 0;
            process_fft(&mut buffer_in, &mut buffer_out);
            // update the output buffer write index to the start of the next hop
            println!("-------- NEW HOP ------------------------");
            buffer_out.next_hop();
        }
        hop_counter += 1;
        writer.write_sample(scaled_out_sample)?;
    }

    writer.finalize()?;
    Ok(())
}

fn process_fft(in_buffer: &mut CircularBuffer<f32, BUFFER_SIZE>, out_buffer: &mut CircularBuffer<f32, BUFFER_SIZE>) {
    let analysis_window_buffer: [f32; FFT_SIZE] = hann_window::HANN_WINDOW;
    let mut unwrapped_buffer: [f32; FFT_SIZE] = [0.0; FFT_SIZE];
    let mut full_spectrum: [microfft::Complex32; FFT_SIZE] =
        [microfft::Complex32 { re: 0.0, im: 0.0 }; FFT_SIZE]; // Full spectrum array

    // copy buffer into FFT input, starting one window ago
    in_buffer.push_read_back(FFT_SIZE - HOP_SIZE);
    for n in 0..FFT_SIZE {
        unwrapped_buffer[n] = in_buffer.read() * analysis_window_buffer[n]
    }

    // Process the FFT based on the time domain input
    let fft = microfft::real::rfft_1024(&mut unwrapped_buffer);

        // Robot the sound 
        let robit_sound =  fft.map(|i|
            {microfft::Complex32 {re: sqrtf( i.re * i.re + i.im  * i.im), im: 0.0}}
         );

    // Reconstruct the full spectrum for the IFFT
    for i in 0..(FFT_SIZE / 2) {
        full_spectrum[i] = robit_sound[i]; // First half directly
        if i > 0 && i < (FFT_SIZE / 2) {
            full_spectrum[FFT_SIZE - i] = robit_sound[i].conj(); // Conjugate symmetry for the second half
        }
    }

    // Run the inverse FFT
    let res = microfft::inverse::ifft_1024(&mut full_spectrum);

    // Add time domain into the output buffer
    for (n, val) in res.iter().enumerate() {
        let windowed_val = val.re * analysis_window_buffer[n]; // Window again and scale
        out_buffer.add_value(windowed_val);
    }
}
