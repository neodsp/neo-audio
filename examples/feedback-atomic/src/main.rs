use std::sync::Arc;

use neo_audio::prelude::*;

pub struct Parameters {
    pub gain: FloatParameter,
}

pub struct FeedbackProcessor {
    params: Arc<Parameters>,
}

impl Default for FeedbackProcessor {
    fn default() -> Self {
        Self {
            params: Arc::new(Parameters {
                gain: FloatParameter::new("gain", 1.0, 0.0..=10.0),
            }),
        }
    }
}

impl AudioProcessor for FeedbackProcessor {
    type Message = ();
    type Parameters = Arc<Parameters>;

    fn prepare(&mut self, config: DeviceConfig) {
        println!("Prepare is called with {:?}", config);
    }

    fn message_process(&mut self, _: Self::Message) {}

    fn process(
        &mut self,
        mut output: InterleavedAudioMut<'_, f32>,
        input: InterleavedAudio<'_, f32>,
    ) {
        for (out_frame, in_frame) in output.frames_iter_mut().zip(input.frames_iter()) {
            out_frame
                .iter_mut()
                .zip(in_frame.iter())
                .for_each(|(o, i)| *o = *i * self.params.gain.value());
        }
    }

    fn parameters(&self) -> Self::Parameters {
        self.params.clone()
    }
}

fn main() -> Result<(), NeoAudioError> {
    // construct audio engine with selected backend and message type
    let mut neo_audio = NeoAudio::<PortAudioBackend>::new()?;

    // start the audio engine with an implemented audio processor
    let (_, params) = neo_audio.start_audio(FeedbackProcessor::default())?;

    params.gain.set_value(0.5);

    std::thread::sleep(std::time::Duration::from_secs(5));

    params.gain.set_value(1.0);

    // let it run for a bit
    std::thread::sleep(std::time::Duration::from_secs(5));

    // stop the audio stream
    neo_audio.stop_audio()?;
    Ok(())
}
