mod circular_buffer;

use circular_buffer::CircularBuffer;
use hound::{Sample, WavReader, WavSpec, WavWriter};
use std::{error::Error, vec};

const FFT_SIZE: usize = 512;
const CHUNK_SIZE: usize = 1024;
fn main() -> Result<(), Box<dyn Error>> {
    let path = "WeChooseToGoToTheMoon.wav";
    let mut reader = WavReader::open(path)?;
    let spec = reader.spec();

    match spec.sample_format {
        hound::SampleFormat::Int => {
            match spec.bits_per_sample {
                8 => read_and_write_samples::<i8>(&mut reader, &spec)?,
                16 => read_and_write_samples::<i16>(&mut reader, &spec)?,
                24 => return Err(Box::from("Unsupport bit depth 24")),
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
    let output_path = "processed_sample.wav";
    let mut writer = WavWriter::create(output_path, *spec)?;
    let mut buffer_in: CircularBuffer<f32> = CircularBuffer::new(0.0, Some(0), Some(0));
    let mut window_in = 0;
    let window_size = 256;
    let hop_size = 128;
    let mut hop_counter = 0;
    let mut buffer_out: CircularBuffer<f32> = CircularBuffer::new(0.0, Some(hop_size), Some(0));

     let i16_samples: Vec<i16> = reader.samples::<i16>().map(|s| s.unwrap()).collect();
    let f32_samples = convert_i16_to_f32(&i16_samples);

    for sample in f32_samples {
        println!("Sample: {:?}", sample);


        buffer_in.write(sample);
        window_in += 1;
        if(window_in >= window_size) {
            window_in = 0;
            // PROCESSES DATA
            println!("Window Size hit")
        }

        let out_sample = buffer_out.read_and_reset();
        let scaled_out_sample = out_sample  * (hop_size as f32)/ FFT_SIZE as f32;
        hop_counter += 1;
        if hop_counter >= hop_size {
            hop_counter = 0;
            process_fft(&mut buffer_in, &mut buffer_out)
            // TODO FFT stuff 

        }

        writer.write_sample(convert_f32_array_to_i16(scaled_out_sample))?;
    }

    writer.finalize()?;
    Ok(())
}

fn process_fft( in_buffer: &mut CircularBuffer<f32>, out_buffer:&mut CircularBuffer<f32>) {
    const BUFFER_SIZE: usize = 1024;
    let analysis_window_buffer: [f32; FFT_SIZE] = hanning_window();
     let mut unwrapped_buffer: [f32; FFT_SIZE] = [0.0; FFT_SIZE];
    let mut fft_output_buffer: [microfft::Complex32; FFT_SIZE / 2 + 1] = [microfft::Complex32 { re: 0.0, im: 0.0 }; FFT_SIZE / 2 + 1];


    // Apply windowing and read from input buffer
    for n in 0..FFT_SIZE {
        unwrapped_buffer[n] = in_buffer.read() * analysis_window_buffer[n] as f32;
    }

    let res = microfft::real::rfft_512(&mut unwrapped_buffer);
    
    // Process FFT output (e.g., calculate amplitude)
    for (i, &complex) in fft_output_buffer.iter().enumerate() {
        let amplitude = (complex.re.powi(2) + complex.im.powi(2)).sqrt(); // Calculate the amplitude
        // println!("Frequency bin {}: amplitude {:?}", i, amplitude);
    }
    

    for n in res.iter().enumerate() {
        let amplitude = n;
        // println!("amplitude {:?}", amplitude);
    }

    // TODO Inverse FFT
    let res = microfft::inverse::ifft_256( res);
    for n in res.iter().enumerate() {
        out_buffer.write(n.1.re)
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

fn convert_i16_to_f32(samples: &[i16]) -> Vec<f32> {
    samples.iter().map(|&sample| sample as f32).collect()
}

fn convert_f32_array_to_i16(value: f32) ->i16 {
        let clamped_value = value.clamp(i16::MIN as f32, i16::MAX as f32);
        clamped_value.round() as i16
}
