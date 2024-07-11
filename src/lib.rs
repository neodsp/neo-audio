use audio_processor::AudioProcessor;
use backends::AudioBackend;
use crossbeam_channel::Sender;
use error::NeoAudioError;

pub mod audio_block;
pub mod audio_processor;
pub mod available_devices;
pub mod backends;
pub mod device_config;
pub mod error;

pub const DEFAULT_SAMPLE_RATES: &[f64] = &[44100.0, 48000.0, 88200.0, 96000.0, 192000.0];
pub const DEFAULT_NUM_FRAMES: &[u16] = &[16, 32, 64, 128, 256, 512, 1024, 2048, 4096];

pub struct NeoAudio<B>
where
    B: AudioBackend,
{
    backend: B,
}

unsafe impl<B> Sync for NeoAudio<B> where B: AudioBackend {}
unsafe impl<B> Send for NeoAudio<B> where B: AudioBackend {}

impl<B> NeoAudio<B>
where
    B: AudioBackend,
{
    pub fn new() -> Result<Self, NeoAudioError> {
        Ok(Self { backend: B::new()? })
    }

    pub fn backend(&self) -> &B {
        &self.backend
    }

    pub fn backend_mut(&mut self) -> &mut B {
        &mut self.backend
    }

    pub fn start_audio<P>(
        &mut self,
        device_config: &crate::device_config::DeviceConfig,
        mut processor: P,
    ) -> Result<Sender<P::Message>, NeoAudioError>
    where
        P: AudioProcessor + Send + 'static,
        <P as audio_processor::AudioProcessor>::Message: std::marker::Send,
    {
        let (sender, receiver) = crossbeam_channel::bounded(1024);
        processor.prepare(device_config);
        self.backend
            .start_stream(device_config, move |input, output| {
                // receive all messages
                for _ in 0..receiver.len() {
                    match receiver.try_recv() {
                        Ok(message) => {
                            processor.message_process(message);
                        }
                        _ => break,
                    }
                }

                // call processor
                processor.process(input, output)?;

                Ok(())
            })?;
        Ok(sender)
    }

    pub fn stop_audio(&mut self) -> Result<(), NeoAudioError> {
        self.backend.stop_stream()?;
        self.backend.stream_error()?;
        Ok(())
    }
}
