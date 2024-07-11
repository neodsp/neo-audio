use thiserror::Error;

#[derive(Debug, Error)]
pub enum NeoAudioError {
    #[error("Device not found {0}")]
    DeviceNotFound(String),
    #[error("Stream has to be build first!")]
    NoStream,
    #[error("Error in Processor {0}")]
    ProcessorError(String),
    #[error("Unknown: {0}")]
    UnknownBackendError(String),
}
