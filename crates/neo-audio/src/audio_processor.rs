pub use crate::device_config::DeviceConfig;
pub use realtime_tools::interleaved_audio::{InterleavedAudio, InterleavedAudioMut};

pub trait AudioProcessor {
    // Messages can be used to send thread-safe messages to the audio process
    // Messages will be updated only once at the beginning of the audio process,
    type Message;

    // parameters can be used to update the audio processor with fast updates
    type Parameters;

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

    // return the parameters that the processor is using
    fn parameters(&self) -> Self::Parameters;
}
