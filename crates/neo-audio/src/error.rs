use system_audio::system_audio_error::SystemAudioError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum NeoAudioError {
    #[error("Sending Message failed")]
    SendFailed,
    #[error("System Audio Error {0}")]
    System(#[from] SystemAudioError),
}
