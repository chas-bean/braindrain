use std::thread;

use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    FromSample, Sample, SampleRate, SizedSample,
};

use braindrain::{waveform, wavetable};
use crossbeam::channel::bounded;

enum MainThreadMessage {
    Sample(f64),
}

enum AudioThreadMessage {
    RequestSample,
}

fn main() {
    let host = cpal::default_host();

    let device = host
        .default_output_device()
        .expect("No default audio device active on host");

    let config = device
        .default_output_config()
        .expect("No default audio output device config");

    // @chas #todo support other non-standard formats i24, i48, etc.
    let handle = thread::spawn(move || match config.sample_format() {
        cpal::SampleFormat::I8 => run::<i8>(&device, &config.into()),
        cpal::SampleFormat::I16 => run::<i16>(&device, &config.into()),
        // cpal::SampleFormat::I24 => run::<I24>(&device, &config.into()),
        cpal::SampleFormat::I32 => run::<i32>(&device, &config.into()),
        // cpal::SampleFormat::I48 => run::<I48>(&device, &config.into()),
        cpal::SampleFormat::I64 => run::<i64>(&device, &config.into()),
        cpal::SampleFormat::U8 => run::<u8>(&device, &config.into()),
        cpal::SampleFormat::U16 => run::<u16>(&device, &config.into()),
        // cpal::SampleFormat::U24 => run::<U24>(&device, &config.into()),
        cpal::SampleFormat::U32 => run::<u32>(&device, &config.into()),
        // cpal::SampleFormat::U48 => run::<U48>(&device, &config.into()),
        cpal::SampleFormat::U64 => run::<u64>(&device, &config.into()),
        cpal::SampleFormat::F32 => run::<f32>(&device, &config.into()),
        cpal::SampleFormat::F64 => run::<f64>(&device, &config.into()),
        sample_format => panic!("Unsupported sample format '{sample_format}'"),
    });

    handle.join().unwrap();
}

pub fn run<T>(device: &cpal::Device, config: &cpal::StreamConfig) -> !
where
    T: SizedSample + FromSample<f32>,
{
    let channels = config.channels as usize;
    let SampleRate(sample_rate) = config.sample_rate;

    let (main_producer, audio_consumer) = bounded(1_000); // @note: Could also use a zero-size?
    let (audio_producer, main_consumer) = bounded(1_000);

    let mut saw_table = wavetable::Wavetable::new(64);

    saw_table.fill(waveform::sawtooth);

    let mut wavetable_iter_220 = saw_table.iter(220.0_f64, sample_rate as f64);

    let mut next_sample = move || {
        audio_producer
            .send(AudioThreadMessage::RequestSample)
            .unwrap();

        match audio_consumer.recv().unwrap() {
            MainThreadMessage::Sample(sample) => sample.to_sample(),
        }
    };

    let stream = device
        .build_output_stream(
            config,
            move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
                write_data(data, channels, &mut next_sample) // @note: Separate thread
            },
            move |err| println!("Stream error: {}", err),
            None,
        )
        .expect("Stream built based on proper config");

    stream.play().expect("Stream is played");

    loop {
        match main_consumer.recv().unwrap() {
            AudioThreadMessage::RequestSample => {
                let sample = wavetable_iter_220.next().unwrap() * 0.1;

                main_producer
                    .send(MainThreadMessage::Sample(sample))
                    .unwrap();
            }
        }
    }
}

fn write_data<T>(output: &mut [T], channels: usize, next_sample: &mut dyn FnMut() -> f32)
where
    T: Sample + FromSample<f32>,
{
    for frame in output.chunks_mut(channels) {
        let value: T = T::from_sample(next_sample());
        for sample in frame.iter_mut() {
            *sample = value;
        }
    }
}
