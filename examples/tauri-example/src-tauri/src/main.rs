// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Mutex;

use neo_audio::{backends::portaudio_backend::PortAudioBackend, prelude::*};

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

pub struct State {
    neo_audio: NeoAudio<PortAudioBackend>,
}

#[tauri::command]
fn get_apis(state: tauri::State<Mutex<State>>) -> Vec<String> {
    state
        .lock()
        .expect("Mutex Posion Error")
        .neo_audio
        .backend()
        .available_apis()
}

#[tauri::command]
fn set_api(state: tauri::State<Mutex<State>>, api_name: &str) -> Result<(), NeoAudioError> {
    state
        .lock()
        .expect("Mutex Posion Error")
        .neo_audio
        .backend_mut()
        .set_api(api_name)?;
    Ok(())
}

fn main() {
    let neo_audio = NeoAudio::new().unwrap();

    tauri::Builder::default()
        .manage(Mutex::new(State { neo_audio }))
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![greet, get_apis, set_api])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

enum MyMessage {
    #[allow(unused)]
    Gain(f32),
}

struct Feedback {
    gain: f32,
}

impl Default for Feedback {
    fn default() -> Self {
        Self { gain: 1.0 }
    }
}

impl AudioProcessor for Feedback {
    type Message = MyMessage;

    fn prepare(&mut self, config: DeviceConfig) {
        println!("Prepare is called with {:?}", config);
    }

    fn message_process(&mut self, message: Self::Message) {
        match message {
            MyMessage::Gain(gain) => self.gain = gain,
        }
    }

    fn process(
        &mut self,
        mut output: InterleavedAudioMut<'_, f32>,
        input: InterleavedAudio<'_, f32>,
    ) {
        let min_ch = output.num_channels().min(input.num_channels());
        for ch in 0..min_ch {
            output
                .channel_iter_mut(ch)
                .zip(input.channel_iter(ch))
                .for_each(|(o, i)| *o = *i * self.gain);
        }
    }
}
