# neo-audio

A backend-agnostic real-time audio engine that works cross platform.

This is still unstable and early work in progress.

## Examples

See the examples folder for all example applications.
For a minimal example check `feedback` and `player`.
For a full settings menu check `egui-example` or `iced-example`.
For integration in a web-view check the `tauri-example`.

```bash
cargo run --bin feedback
# or
cargo run --bin player
# or
cargo run --bin egui-example
# or
cargo run --bin iced-example
# or
cd examples/tauri-example/
npm install
npm run tauri dev
```

## Usage

To include it in your project add this to your `Cargo.toml` file:

```toml
[dependencies]
neo-audio = { git = "https://github.com/neodsp/neo-audio", tag = "0.2.1" }
```

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
    fn process(
        &mut self,
        mut output: InterleavedAudioMut<'_, f32>,
        input: InterleavedAudio<'_, f32>,
    ) {
        for (out_frame, in_frame) in output.frames_iter_mut().zip(input.frames_iter()) {
            out_frame
                .iter_mut()
                .zip(in_frame.iter())
                .for_each(|(o, i)| *o = *i * self.gain);
        }
    }
}
```

Create an instance with a specific backend. Here we will use the PortAudio backend.

```Rust
let mut neo_audio = NeoAudio::<PortAudioBackend>::new()?;
```

You can get and set the available devices and settings on the system via the `backend` function.
For a list of all available functions check the `AudioBackend` trait.

```Rust
let output_devices = neo_audio.backend().available_output_devices();

// don't start an output stream
neo_audio.backend_mut().set_output_device(Device::None)?;
// use the system default device
neo_audio.backend_mut().set_output_device(Device::Default)?;
// specify a device by name
neo_audio
    .backend_mut()
    .set_output_device(Device::Name("My Soundcard Name".into()))?;

let _selected_output_device = neo_audio.backend().output_device();
```

Start the audio stream with the selected settings. You have to call the constructor of the Processor here manually, to increase flexibility.
The function will return a sender that can be cloned as often as you like to send messages to the audio thread.

```Rust
let sender = neo_audio.start_audio(MyProcessor::default())?;
```

Send a message to the audio callback.

```Rust
sender.send(MyMessage::Gain(0.5))?;
```

Stop the audio stream.

```Rust
neo_audio.stop_audio()?;
```

## Prerequisites

For RtAudio Backend install the following dependencies:

### Fedora Linux

```bash
sudo dnf install cmake alsa-lib-devel pulseaudio-libs-devel
```

### Ubuntu Linux

```bash
sudo apt-get install cmake libasound2-dev libpulse-dev
```
For some examples you need to install the Prerequisited of the UI frameworks

- iced: https://github.com/iced-rs/iced/blob/master/DEPENDENCIES.md
- Tauri: https://tauri.app/start/prerequisites/
