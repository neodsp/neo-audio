# neo-audio

A backend-agnostic real-time audio engine that works cross platform.

## Examples

See the examples folder for all example applications.
For a minimal example check `feedback` and `player`.
For a full settings menu check `egui-example`.

```bash
cargo run --bin feedback
# or
cargo run --bin player
# or
cargo run --bin egui-example
```

## Usage

Import all necessary functions to use the engine

```Rust
use neo_audio::prelude::*;
```

Create a Message Type and implement the `AudioProcessor` trait or use one of the pre-defined processors behind the `processors` feature-flag like `PlayerProcessor` and `FeedbackProcesssor`.

```Rust
enum MyMessage {
    Gain(f32),
}

struct MyProcessor {
    gain: f32,
}

impl Default for MyProcessor {
    fn default() -> Self {
        Self { gain: 1.0 }
    }
}

impl AudioProcessor for MyProcessor {
    type Message = MyMessage;

    fn prepare(&mut self, config: DeviceConfig) {
        println!("Prepare is called with {:?}", config);
    }

    fn message_process(&mut self, message: Self::Message) {
        match message {
            MyMessage::Gain(gain) => self.gain = gain,
        }
    }

    /// This is a simple feedback with gain
    fn process(&mut self, mut output: InterleavedAudioMut<'_, f32>, input: InterleavedAudio<'_, f32>) {
        let min_ch = output.num_channels().min(input.num_channels());
        for ch in 0..min_ch {
            output
                .channel_iter_mut(ch)
                .zip(input.channel_iter(ch))
                .for_each(|(o, i)| *o = *i * self.gain);
        }
    }
}
```

Create an instance with a specific backend and processor. Here we will use the RtAudio backend.

```Rust
let mut neo_audio = NeoAudio::<RtAudioBackend, MyProcessor>::new()?;
```

You can get and set the available devices and settings on the system via the `backend` function.
For a list of all available functions check the `AudioBackend` trait.

```Rust
    let output_devices = neo_audio.backend().available_output_devices();

    // don't start an output stream
    neo_audio.backend().set_output_device(AudioDevice::None)?;
    // use the system default device
    neo_audio.backend().set_output_device(AudioDevice::Default)?;
    // specify a device by name
    neo_audio.backend().set_output_device(AudioDevice::Name("My Soundcard Name"))?;

    let selected_output_device = neo_audio.backend().output_device();
```

Start the audio stream with the selected settings. You have to call the constructor of the Processor here manually, to increase flexibility.

```Rust
neo_audio.start_audio(MyProcessor::default())?;
```

Send a message to the audio callback.

```Rust
neo_audio.send_message(MyMessage::Gain(0.5))?;
```

Stop the audio stream.

```Rust
neo_audio.stop_audio()?;
```
