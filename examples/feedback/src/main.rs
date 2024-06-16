use neo_audio::{
    backends::portaudio_backend::PortAudioBackend, prelude::*, processors::feedback::*,
};

fn main() -> Result<(), Error> {
    // construct audio engine with selected backend and message type
    let mut neo_audio = NeoAudio::<PortAudioBackend>::new()?;

    // start the audio engine with an implemented audio processor
    let sender = neo_audio.start_audio(FeedbackProcessor::default())?;

    // send thread-safe messages to the processor
    sender.send(FeedbackMessage::Gain(0.5)).unwrap();

    // let it run for a bit
    std::thread::sleep(std::time::Duration::from_secs(5));

    // stop the audio stream
    neo_audio.stop_audio()?;
    Ok(())
}
