use crate::prelude::*;

pub enum FeedbackMessage {
    /// Gain will set the gain in linear values.
    /// - 0.0 will turn off the audio completely
    /// - 1.0 will play the audio in original volume
    /// - 2.0 will play the audio 6dB louder
    Gain(f32),
}

pub struct FeedbackProcessor {
    gain: f32,
}

impl Default for FeedbackProcessor {
    fn default() -> Self {
        Self { gain: 1.0 }
    }
}

impl AudioProcessor for FeedbackProcessor {
    type Message = FeedbackMessage;

    fn prepare(&mut self, config: DeviceConfig) {
        println!("Prepare is called with {:?}", config);
    }

    fn message_process(&mut self, message: Self::Message) {
        match message {
            FeedbackMessage::Gain(gain) => self.gain = gain,
        }
    }

    fn process(&mut self, mut output: AudioDataMut<'_, f32>, input: AudioData<'_, f32>) {
        for (out_frame, in_frame) in output.frames_iter_mut().zip(input.frames_iter()) {
            out_frame
                .iter_mut()
                .zip(in_frame.iter())
                .for_each(|(o, i)| *o = *i * self.gain);
        }
    }
}
