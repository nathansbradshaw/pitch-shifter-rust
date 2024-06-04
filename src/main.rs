mod circular_buffer;

use hound::{WavReader, WavSpec, WavWriter};
use std::error::Error;

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

    for sample in reader.samples::<S>() {
        let sample = sample?;
        println!("Sample: {:?}", sample);

        // Here you can add your processing logic
        let processed_sample = sample;

        writer.write_sample(processed_sample)?;
    }

    writer.finalize()?;
    Ok(())
}
