use neo_audio::{backends::portaudio_backend::PortAudioBackend, prelude::*, processors::player::*};

fn generate_sine_wave(freq: f32, sample_rate: u32, duration: f32) -> Vec<f32> {
    let num_samples = (sample_rate as f32 * duration) as usize;
    let t: Vec<f32> = (0..num_samples)
        .map(|i| i as f32 / sample_rate as f32)
        .collect();
    t.iter()
        .map(|&x| (2.0 * std::f32::consts::PI * freq * x).sin())
        .collect()
}

fn main() -> Result<(), NeoAudioError> {
    // construct audio engine with selected backend and message type
    let mut neo_audio = NeoAudio::<PortAudioBackend>::new()?;

    // generate stereo sine
    let sine_left = generate_sine_wave(440.0, neo_audio.backend().sample_rate(), 1.0);
    let sine_right = generate_sine_wave(600.0, neo_audio.backend().sample_rate(), 1.0);
    let mut stereo_sine = Array2::default((2, sine_left.len()));
    sine_left.iter().enumerate().for_each(|(i, v)| {
        stereo_sine[[0, i]] = *v;
    });
    sine_right.iter().enumerate().for_each(|(i, v)| {
        stereo_sine[[1, i]] = *v;
    });

    // generate channel
    let (sender, receiver) = crossbeam_channel::bounded(1024);

    // start the audio engine with an implemented audio processor
    let mut player = PlayerProcessor::default();
    player.set_audio(stereo_sine);
    player.set_progress_sender(sender);
    let sender = neo_audio.start_audio(player)?;

    // send thread-safe messages to the processor
    sender.send(PlayerMessage::Play).unwrap();
    sender.send(PlayerMessage::Gain(0.5)).unwrap();

    // let it run until the whole file was played
    'outer: loop {
        for _ in 0..receiver.len() {
            match receiver.try_recv() {
                Ok(progress) => {
                    println!("Progress {}%", (progress * 100.0) as usize);
                    if progress == 1.0 {
                        break 'outer;
                    }
                }
                _ => break,
            }
        }
    }

    // stop the audio stream
    neo_audio.stop_audio()?;

    Ok(())
}
