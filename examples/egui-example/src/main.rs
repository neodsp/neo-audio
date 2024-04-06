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
    neo_audio: NeoAudio<SystemRtAudio, MyMessage>,
    audio_running: bool,
    api: String,
    output_device: Option<String>,
}

impl MyEguiApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        let neo_audio = NeoAudio::<SystemRtAudio, MyMessage>::new().unwrap();
        let system_audio = neo_audio.system_audio();
        Self {
            audio_running: false,
            api: system_audio.api(),
            output_device: system_audio.output_device(),
            neo_audio,
        }
    }
}

impl eframe::App for MyEguiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_pixels_per_point(1.5);
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("NeoAudio egui example!");

            let system_audio = self.neo_audio.system_audio_mut();

            egui::ComboBox::from_label("Api")
                .selected_text(system_audio.api())
                .show_ui(ui, |ui| {
                    for api in system_audio.available_apis() {
                        ui.selectable_value(&mut self.api, api.clone(), api);
                    }
                });

            // update if changed
            if self.api != system_audio.api() {
                system_audio.set_api(&self.api).unwrap();
            }

            egui::ComboBox::from_label("Output Device")
                .selected_text(format!("{:?}", system_audio.output_device()))
                .show_ui(ui, |ui| {
                    for device in system_audio.available_output_devices() {
                        ui.selectable_value(&mut self.output_device, Some(device.clone()), device);
                    }
                });

            // update if changed
            if self.output_device != system_audio.output_device() {
                system_audio
                    .set_output_device(self.output_device.as_ref().into())
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
