use crossbeam_channel::SendError;

#[derive(thiserror::Error, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum NeoAudioError {
    #[error("Stream is running, please stop stream before changing devices.")]
    StreamRunning,
    #[error("Api not found")]
    ApiNotFound,
    #[error("Output Device not found")]
    OutputDeviceNotFound,
    #[error("Input Device not found")]
    InputDeviceNotFound,
    #[error("Samplerate not supported")]
    SampleRate,
    #[error("Number of frames not supported")]
    NumFrames,
    #[error("Number of input channels not supported")]
    NumInputChannels,
    #[error("Number of output channels not supported")]
    NumOutputChannels,
    #[error("Unspecified")]
    Unspecified,
    #[error("Backend Error {0}")]
    Backend(String),
    #[error("Failed to start stream")]
    StartStream(String),
    #[error("Failed to open stream with config")]
    OpenStream(String),
    #[error("Failed to send message containing {0}")]
    Send(String),
}

impl<T> From<SendError<T>> for NeoAudioError {
    fn from(s: SendError<T>) -> Self {
        NeoAudioError::Send(s.to_string())
    }
}
