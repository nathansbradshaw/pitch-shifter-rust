mod circular_buffer;

use circular_buffer::CircularBuffer;
use hound::{ WavReader, WavSpec, WavWriter};
use std::error::Error;

const FFT_SIZE: usize = 1024;
fn main() -> Result<(), Box<dyn Error>> {
    let path = "sample_f32.wav";
    // let path = "WeChooseToGoToTheMoon_f32.wav";
    let mut reader = WavReader::open(path)?;
    let spec = reader.spec();

    match spec.sample_format {
        hound::SampleFormat::Int => {
            match spec.bits_per_sample {
                _ => return Err(Box::from("Unsupported bit depth")),
            }
        },
        hound::SampleFormat::Float => {
            match spec.bits_per_sample {
                32 => read_and_write_samples::<f32>(&mut reader, &spec)?,
                _ => return Err(Box::from("Unsupported bit depth")),
            }
        },
    }

    Ok(())
}

fn read_and_write_samples<S>(
    reader: &mut WavReader<std::io::BufReader<std::fs::File>>,
    spec: &WavSpec
) -> Result<(), Box<dyn Error>>
where
    S: hound::Sample + std::fmt::Debug + hound::Sample,
{
    let output_spec = WavSpec {
        ..*spec
    };

    let output_path = "processed_sample.wav";
    let mut writer = WavWriter::create(output_path, output_spec)?;
    let mut buffer_in: CircularBuffer<f32> = CircularBuffer::new(0.0, Some(0));
    let hop_size = 1000;
    let mut hop_counter = 0;
    let mut buffer_out: CircularBuffer<f32> = CircularBuffer::new(0.0, Some(hop_size));
    

    for sample in reader.samples::<f32>() {
        let sample = sample.expect("Error reading sample");
        println!("Sample: {:?}", sample);


        // Store the sample in the input buffer
        buffer_in.write(sample);

        // Read from the output buffer and reset the value
        let out_sample = buffer_out.read_and_reset();

        // Scale the output dow by the overlap factor
        let scaled_out_sample = out_sample  * (hop_size as f32)/ FFT_SIZE as f32;

        // Increment the hop counter
        if hop_counter >= hop_size {
            hop_counter = 0;
            process_fft(&mut buffer_in, &mut buffer_out);
            // update the output buffer write index to the start of the next hop
            buffer_out.next_hop();
        }
        hop_counter += 1;
        writer.write_sample(scaled_out_sample)?;

    }

    writer.finalize()?;
    Ok(())
}

fn process_fft( in_buffer: &mut CircularBuffer<f32>, out_buffer:&mut CircularBuffer<f32>) {
    let analysis_window_buffer: [f32; FFT_SIZE] = hanning_window();
     let mut unwrapped_buffer: [f32; FFT_SIZE] = [0.0; FFT_SIZE];
     let mut full_spectrum: [microfft::Complex32; FFT_SIZE] = [microfft::Complex32 { re: 0.0, im: 0.0 }; FFT_SIZE]; // Full spectrum array



     // copy buffer into FFT input, starting one window ago
     in_buffer.push_read_back(FFT_SIZE);
     for n in 0..FFT_SIZE {
        unwrapped_buffer[n] = in_buffer.read()
     }


     // Process the FFT based on the time domain input
    let fft = microfft::real::rfft_1024(&mut unwrapped_buffer);
    

        // Reconstruct the full spectrum for the IFFT
        for i in 0..(FFT_SIZE / 2 ) {
            full_spectrum[i] = fft[i]; // First half directly
            if i > 0 && i < (FFT_SIZE / 2) {
                full_spectrum[FFT_SIZE - i] = fft[i].conj(); // Conjugate symmetry for the second half
            }
        }
    
     // Run the inverse FFT 
     let res = microfft::inverse::ifft_1024(&mut full_spectrum);

    // Add time domain into the output buffer 
    for n in res.iter().enumerate() {
        out_buffer.add_value(n.1.re)
    }
}


// Function to generate a Hanning window
fn hanning_window() -> [f32; FFT_SIZE] {
    let mut window = [0.0; FFT_SIZE];
    for n in 0..512 {
        window[n] = 0.5 * (1.0 - (2.0 *  3.14 * n as f32 / (FFT_SIZE - 1) as f32).cos());
    }
    window
}
