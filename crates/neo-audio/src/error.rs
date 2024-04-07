use audio_backend::audio_backend_error::AudioBackendError;
use thiserror::Error;

#[derive(Error, Debug, serde::Serialize, serde::Deserialize)]
pub enum NeoAudioError {
    #[error("Sending Message failed")]
    SendFailed,
    #[error("System Audio Error {0}")]
    System(#[from] AudioBackendError),
}
