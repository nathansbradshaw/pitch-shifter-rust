# Rust Vocoder
This is a simple side project to learn how to build a vocoder. The choice to stay away from using standard library methods was intentional since this is built with the intention to use in an embedded application.
## How to run
- [Install Rust](https://rustup.rs/)
- `Cargo Run`

This vocoder only works on Wav files with an f32 format. Changing pitch, input file or output file currently can only be done by updating some variables in main.

`PITCH_SHIFT` changes the pitch of the audio sample. a value of `1` is normal

`path` denotes the input file path

`output_path` tells what the output file will be.

`HOP_SIZE` is another number worth playing with, it determines how frequently the samples are processed. When set to `128` The hop size is 1/8 of the window (FFT_SIZE), Hop_size should always be smaller than window sizes and a clean division 1/2, 1/4, 1/8, etc.
