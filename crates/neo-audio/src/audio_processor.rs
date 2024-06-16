pub use audio_backend::device_config::DeviceConfig;
pub use rt_tools::interleaved_audio::{InterleavedAudio, InterleavedAudioMut};

pub trait AudioProcessor {
    type Message;

    /// prepare is called just before the audio engine is started, so that anything can be handled
    /// that is not "real-time" safe, like resize arrays, make system calls etc.
    fn prepare(&mut self, config: DeviceConfig);

    /// the message process will handle all incoming messages in the audio thread.
    /// beware that it is not recommended to do anything computation heavy or anything that is not
    /// regarded "real-time-safe". Usually this is used to copy small values like floats and bools
    /// to the processor.
    fn message_process(&mut self, message: Self::Message);

    /// here you can manipulate the audio streams, copy incoming to outgoing data.
    /// do not do anything that blocks the audio stream.
    fn process(&mut self, output: InterleavedAudioMut<'_, f32>, input: InterleavedAudio<'_, f32>);

    // here you can reset stuff if it the audio is stopped
    fn stopped(&mut self);
}
