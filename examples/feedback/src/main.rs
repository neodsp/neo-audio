use neo_audio::prelude::*;

fn main() -> Result<(), NeoAudioError> {
    // construct audio engine with selected backend and message type
    let mut neo_audio = NeoAudio::<RtAudioBackend, MyProcessor>::new()?;

    // start the audio engine with an implemented audio processor
    neo_audio.start_audio(MyProcessor::default())?;

    // send thread-safe messages to the processor
    neo_audio.send_message(MyMessage::Gain(0.5))?;

    // let it run for a bit
    std::thread::sleep(std::time::Duration::from_secs(5));

    // stop the audio stream
    neo_audio.stop_audio()?;
    Ok(())
}

enum MyMessage {
    Gain(f32),
}

struct MyProcessor {
    gain: f32,
}

impl Default for MyProcessor {
    fn default() -> Self {
        Self { gain: 1.0 }
    }
}

impl AudioProcessor for MyProcessor {
    type Message = MyMessage;

    fn prepare(&mut self, config: DeviceConfig) {
        println!("Prepare is called with {:?}", config);
    }

    fn message_process(&mut self, message: Self::Message) {
        match message {
            MyMessage::Gain(gain) => self.gain = gain,
        }
    }

    fn process(
        &mut self,
        mut output: InterleavedAudioMut<'_, f32>,
        input: InterleavedAudio<'_, f32>,
    ) {
        let min_ch = output.num_channels().min(input.num_channels());
        for ch in 0..min_ch {
            output
                .channel_iter_mut(ch)
                .zip(input.channel_iter(ch))
                .for_each(|(o, i)| *o = *i * self.gain);
        }
    }
}
