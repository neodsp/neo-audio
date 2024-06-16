pub use crossbeam_channel::{self, *};
pub use ndarray::prelude::*;

use crate::prelude::AudioProcessor;

pub enum PlayerMessage {
    /// PLay will play the audio once until the end, or until you send pause/stop
    Play,
    /// PlayLoop will play the audio in a never ending loop, until you send pause or stop
    PlayLoop,
    /// Pause will pause the playback, but won't reset the play-head, so the next time you send
    /// play it will continue to play from the last position
    Pause,
    /// Stop will pause the playback and reset the play-head, so the next time you send play it
    /// will play from the beginning
    Stop,
    /// Gain will set the gain in linear values.
    /// - 0.0 will turn off the audio completely
    /// - 1.0 will play the audio in original volume
    /// - 2.0 will play the audio 6dB louder
    Gain(f32),
}

pub struct PlayerProcessor {
    audio: Array2<f32>,
    play_head: usize,
    play: bool,
    looped: bool,
    progress_sender: Option<Sender<f32>>,
    gain: f32,
}

impl Default for PlayerProcessor {
    fn default() -> Self {
        Self {
            audio: Array2::zeros((0, 0)),
            play_head: 0,
            play: false,
            looped: false,
            progress_sender: None,
            gain: 1.0,
        }
    }
}

impl PlayerProcessor {
    pub fn set_audio(&mut self, audio: impl Into<Array2<f32>>) {
        self.audio = audio.into();
    }

    pub fn set_progress_sender(&mut self, sender: Sender<f32>) {
        self.progress_sender = Some(sender);
    }
}

impl AudioProcessor for PlayerProcessor {
    type Message = PlayerMessage;

    fn prepare(&mut self, _config: audio_backend::device_config::DeviceConfig) {}

    fn message_process(&mut self, message: Self::Message) {
        match message {
            PlayerMessage::Play => {
                self.play = true;
            }
            PlayerMessage::PlayLoop => {
                self.play = true;
                self.looped = true;
            }
            PlayerMessage::Pause => {
                self.play = false;
            }
            PlayerMessage::Stop => {
                self.play = false;
                self.play_head = 0;
            }
            PlayerMessage::Gain(gain) => {
                self.gain = gain;
            }
        }
    }

    fn process(
        &mut self,
        mut output: rt_tools::interleaved_audio::InterleavedAudioMut<'_, f32>,
        _input: rt_tools::interleaved_audio::InterleavedAudio<'_, f32>,
    ) {
        let play_ch = output.num_channels().min(self.audio.nrows());
        for frame in output.frames_iter_mut() {
            if self.play && self.play_head < self.audio.ncols() {
                #[allow(clippy::needless_range_loop)]
                for ch in 0..play_ch {
                    frame[ch] = self.audio[[ch, self.play_head]] * self.gain;
                }
                self.play_head += 1;

                // send progress non-blocking if sender is set
                if let Some(sender) = self.progress_sender.as_ref() {
                    sender
                        .try_send(self.play_head as f32 / self.audio.ncols() as f32)
                        .unwrap_or_default(); // ignore send error
                }
            } else {
                frame.fill(0.0);
            }
        }

        // if audio was completely played, reset playhead
        // and stop audio if audio should not be looped
        if self.play_head >= self.audio.ncols() {
            self.play_head = 0;
            self.play = self.looped;
        }
    }

    fn stopped(&mut self) {
        self.play_head = 0;
        self.play = false;
    }
}
