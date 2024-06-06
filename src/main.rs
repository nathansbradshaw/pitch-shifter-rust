mod circular_buffer;

use circular_buffer::CircularBuffer;
use hound::{WavReader, WavSpec, WavWriter};
use std::error::Error;

const CHUNK_SIZE: usize = 1024;
fn main() -> Result<(), Box<dyn Error>> {
    let path = "sample.wav";
    let mut reader = WavReader::open(path)?;
    let spec = reader.spec();

    match spec.sample_format {
        hound::SampleFormat::Int => {
            match spec.bits_per_sample {
                8 => read_and_write_samples::<i8>(&mut reader, &spec)?,
                16 => read_and_write_samples::<i16>(&mut reader, &spec)?,
                24 => read_and_write_samples::<i32>(&mut reader, &spec)?,
                32 => read_and_write_samples::<i32>(&mut reader, &spec)?,
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
    let mut buffer_in: CircularBuffer<i16> = CircularBuffer::new(0, Some(0), Some(0));
    let mut window_in = 0;
    let window_size = 256;
    let hop_size = 128;
    let fft_size = 64;
    let mut hop_counter = 0;
    let mut buffer_out: CircularBuffer<i16> = CircularBuffer::new(0, Some(hop_size), Some(0));

    for sample in reader.samples::<S>() {
        let sample = sample?.as_i16();
        println!("Sample: {:?}", sample);


        buffer_in.write(sample);
        window_in += 1;
        // if(window_in >= window_size) {
        //     window_in = 0;
        //     // PROCESSES DATA
        //     println!("Window Size hit")
        // }

        let out_sample = buffer_out.read_and_reset();
        // let scaled_out_sample = out_sample  * (hop_size as i16)/ fft_size;
        hop_counter += 1;
        if hop_counter >= hop_size {
            hop_counter = 0;
            process_fft(&mut buffer_in, &mut buffer_out)
            // TODO FFT stuff 

        }




        // Here you can add your processing logic
        let processed_sample = sample;

        writer.write_sample(buffer_out.read())?;
    }

    writer.finalize()?;
    Ok(())
}

fn process_fft( in_buffer: &mut CircularBuffer<i16>, out_buffer:&mut CircularBuffer<i16>) {
    const FFT_SIZE: usize = 512;
    const BUFFER_SIZE: usize = 1024;
    let analysis_window_buffer = [1; FFT_SIZE];
    let mut unwrapped_buffer = [0.0; FFT_SIZE];

    // TODO FFT stuff for REAL
    for n in 0..FFT_SIZE {
        unwrapped_buffer[n] = (in_buffer.read() * analysis_window_buffer[n]) as f32; 
    }

    let res = microfft::real::rfft_512(&mut unwrapped_buffer);

    for n in res.iter().enumerate() {
        let amplitude = n;
        println!("amplitude {:?}", amplitude);
    }

    // TODO Inverse FFT
    let res = microfft::inverse::ifft_256( res);
    for n in res.iter().enumerate() {
        out_buffer.write(n.1.re as i16)
    }
}