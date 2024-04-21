use neo_audio::{prelude::*, processors::feedback::*};

fn main() -> Result<(), NeoAudioError> {
    // construct audio engine with selected backend and message type
    let mut neo_audio = NeoAudio::<RtAudioBackend, FeedbackProcessor>::new()?;

    // start the audio engine with an implemented audio processor
    neo_audio.start_audio(FeedbackProcessor::default())?;

    // send thread-safe messages to the processor
    neo_audio.send_message(FeedbackMessage::Gain(0.5))?;

    // let it run for a bit
    std::thread::sleep(std::time::Duration::from_secs(5));

    // stop the audio stream
    neo_audio.stop_audio()?;
    Ok(())
}
