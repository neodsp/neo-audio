use eframe::egui;
use neo_audio::prelude::*;

fn main() {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "My egui App",
        native_options,
        Box::new(|cc| Box::new(MyEguiApp::new(cc))),
    )
    .unwrap();
}

struct MyEguiApp {
    neo_audio: NeoAudio<RtAudioBackend, MyMessage>,
    audio_running: bool,
    config: DeviceConfig,
    gain: f32,
}

impl MyEguiApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        let neo_audio = NeoAudio::<RtAudioBackend, MyMessage>::new().unwrap();
        let system_audio = neo_audio.system_audio();
        dbg!(system_audio);
        Self {
            audio_running: false,
            config: system_audio.config(),
            neo_audio,
            gain: 1.0,
        }
    }
}

impl eframe::App for MyEguiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_pixels_per_point(1.5);
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("NeoAudio egui example!");

            let system_audio = self.neo_audio.system_audio();

            // API
            egui::ComboBox::from_label("Api")
                .selected_text(system_audio.api())
                .show_ui(ui, |ui| {
                    for api in system_audio.available_apis() {
                        ui.selectable_value(&mut self.config.api, api.clone(), api);
                    }
                });

            // Output Device
            egui::ComboBox::from_label("Output Device")
                .selected_text(format!(
                    "{:?}",
                    system_audio.output_device().unwrap_or("None".to_string())
                ))
                .show_ui(ui, |ui| {
                    for device in system_audio.available_output_devices() {
                        ui.selectable_value(
                            &mut self.config.output_device,
                            DeviceName::Name(device.clone()),
                            device,
                        );
                    }
                });
            egui::ComboBox::from_label("Num Output Channels")
                .selected_text(format!("{:?}", system_audio.num_output_channels()))
                .show_ui(ui, |ui| {
                    for ch in 0..system_audio.available_num_output_channels() {
                        ui.selectable_value(&mut self.config.num_output_ch, ch, ch.to_string());
                    }
                });

            // Input Device
            egui::ComboBox::from_label("Input Device")
                .selected_text(format!(
                    "{:?}",
                    system_audio.input_device().unwrap_or("None".to_string())
                ))
                .show_ui(ui, |ui| {
                    for device in system_audio.available_input_devices() {
                        ui.selectable_value(
                            &mut self.config.input_device,
                            DeviceName::Name(device.clone()),
                            device,
                        );
                    }
                });

            egui::ComboBox::from_label("Num Input Channels")
                .selected_text(format!("{:?}", system_audio.num_input_channels()))
                .show_ui(ui, |ui| {
                    for ch in 0..system_audio.available_num_input_channels() {
                        ui.selectable_value(&mut self.config.num_input_ch, ch, ch.to_string());
                    }
                });

            // Sample Rate
            egui::ComboBox::from_label("Sample Rate")
                .selected_text(format!("{}", system_audio.sample_rate()))
                .show_ui(ui, |ui| {
                    for sr in system_audio.available_sample_rates() {
                        ui.selectable_value(&mut self.config.sample_rate, sr, sr.to_string());
                    }
                });

            // Num Frames
            egui::ComboBox::from_label("Num Frames")
                .selected_text(format!("{}", system_audio.num_frames()))
                .show_ui(ui, |ui| {
                    for frames in system_audio.available_num_frames() {
                        ui.selectable_value(
                            &mut self.config.num_frames,
                            frames,
                            frames.to_string(),
                        );
                    }
                });

            if self.config != system_audio.config() {
                if self.audio_running {
                    self.neo_audio.stop_audio().unwrap();
                    self.audio_running = false;
                }
                self.neo_audio
                    .system_audio_mut()
                    .set_config(&self.config)
                    .unwrap();
            }

            #[allow(clippy::collapsible_else_if)]
            if self.audio_running {
                if ui.button("Stop").clicked() {
                    self.neo_audio.stop_audio().unwrap();
                    self.audio_running = false;
                }
            } else {
                if ui.button("Start").clicked() {
                    self.neo_audio.start_audio(MyProcessor::default()).unwrap();
                    self.audio_running = true;
                }
            }

            if ui
                .add(egui::Slider::new(&mut self.gain, 0.0..=1.0).text("Gain"))
                .changed()
            {
                self.neo_audio
                    .send_message(MyMessage::Gain(self.gain))
                    .unwrap();
            }
        });
    }
}

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

impl AudioProcessor<MyMessage> for MyProcessor {
    fn prepare(&mut self, config: DeviceConfig) {
        println!("Prepare is called with {:?}", config);
    }

    fn message_process(&mut self, message: MyMessage) {
        match message {
            MyMessage::Gain(gain) => self.gain = gain,
        }
    }

    fn process(&mut self, mut output: OutputBuffer<'_, f32>, input: InputBuffer<'_, f32>) {
        let min_ch = output.num_channels().min(input.num_channels());
        for ch in 0..min_ch {
            output
                .channel_iter_mut(ch)
                .zip(input.channel_iter(ch))
                .for_each(|(o, i)| *o = *i * self.gain);
        }
    }
}
