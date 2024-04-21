use eframe::egui;
use level_meter::level_meter;
use neo_audio::{
    prelude::*,
    processors::player::{bounded, Receiver, Sender},
};
use rt_tools::{
    level_meter::{Level, LevelMeter},
    smooth_value::{Easing, Linear, SmoothValue},
};

mod level_meter;

fn main() {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "neo-audio egui example",
        native_options,
        Box::new(|cc| Box::new(NeoAudioEguiExample::new(cc))),
    )
    .unwrap();
}

struct NeoAudioEguiExample {
    neo_audio: NeoAudio<RtAudioBackend, MyProcessor>,
    audio_running: bool,
    config: DeviceConfig,
    gain: f32,
    ui_sender: Sender<UiMessage>,
    ui_receiver: Receiver<UiMessage>,
    input_level: SmoothValue,
}

impl NeoAudioEguiExample {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        let neo_audio = NeoAudio::<RtAudioBackend, MyProcessor>::new().unwrap();
        let backend = neo_audio.backend();
        let (ui_sender, ui_receiver) = bounded(1024);
        let mut input_level = SmoothValue::new(-60.0, Linear::ease_in_out);
        input_level.prepare(60, 100);
        Self {
            audio_running: false,
            config: backend.config(),
            neo_audio,
            gain: 1.0,
            ui_sender,
            ui_receiver,
            input_level,
        }
    }
}

impl eframe::App for NeoAudioEguiExample {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_pixels_per_point(2.0);
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("neo-audio egui example!");

            let backend = self.neo_audio.backend();

            // API
            egui::ComboBox::from_label("Api")
                .selected_text(backend.api())
                .show_ui(ui, |ui| {
                    for api in backend.available_apis() {
                        ui.selectable_value(&mut self.config.api, api.clone(), api);
                    }
                });

            // Output Device
            egui::ComboBox::from_label("Output Device")
                .selected_text(format!(
                    "{:?}",
                    backend.output_device().unwrap_or("None".to_string())
                ))
                .show_ui(ui, |ui| {
                    for device in backend.available_output_devices() {
                        ui.selectable_value(
                            &mut self.config.output_device,
                            Device::Name(device.clone()),
                            device,
                        );
                    }
                });
            egui::ComboBox::from_label("Num Output Channels")
                .selected_text(format!("{:?}", backend.num_output_channels()))
                .show_ui(ui, |ui| {
                    for ch in 1..=backend.available_num_output_channels() {
                        ui.selectable_value(&mut self.config.num_output_ch, ch, ch.to_string());
                    }
                });

            // Input Device
            egui::ComboBox::from_label("Input Device")
                .selected_text(format!(
                    "{:?}",
                    backend.input_device().unwrap_or("None".to_string())
                ))
                .show_ui(ui, |ui| {
                    for device in backend.available_input_devices() {
                        ui.selectable_value(
                            &mut self.config.input_device,
                            Device::Name(device.clone()),
                            device,
                        );
                    }
                });

            egui::ComboBox::from_label("Num Input Channels")
                .selected_text(format!("{:?}", backend.num_input_channels()))
                .show_ui(ui, |ui| {
                    for ch in 1..=backend.available_num_input_channels() {
                        ui.selectable_value(&mut self.config.num_input_ch, ch, ch.to_string());
                    }
                });

            // Sample Rate
            egui::ComboBox::from_label("Sample Rate")
                .selected_text(format!("{}", backend.sample_rate()))
                .show_ui(ui, |ui| {
                    for sr in backend.available_sample_rates() {
                        ui.selectable_value(&mut self.config.sample_rate, sr, sr.to_string());
                    }
                });

            // Num Frames
            egui::ComboBox::from_label("Num Frames")
                .selected_text(format!("{}", backend.num_frames()))
                .show_ui(ui, |ui| {
                    for frames in backend.available_num_frames() {
                        ui.selectable_value(
                            &mut self.config.num_frames,
                            frames,
                            frames.to_string(),
                        );
                    }
                });

            if self.config != backend.config() {
                if self.audio_running {
                    self.neo_audio.stop_audio().unwrap();
                    self.audio_running = false;
                }
                // update config and receive actually applied config
                self.config = self
                    .neo_audio
                    .backend_mut()
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
                    self.neo_audio
                        .start_audio(MyProcessor::new(self.ui_sender.clone()))
                        .unwrap();
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

            // update percentage
            if self.audio_running {
                for _ in 0..self.ui_receiver.len() {
                    match self.ui_receiver.try_recv() {
                        Ok(message) => match message {
                            UiMessage::Level(level) => {
                                self.input_level.set_target_value(level.peak_db);
                            }
                        },
                        _ => break,
                    }
                }
                ui.ctx().request_repaint();
            } else {
                self.input_level.set_current_and_target_value(-60.0);
            }

            ui.add(level_meter(-60.0..=10.0, self.input_level.next_value()));
        });
    }
}

enum MyMessage {
    Gain(f32),
}

enum UiMessage {
    Level(Level),
}

struct MyProcessor {
    gain: SmoothValue,
    meter: LevelMeter,
}

impl MyProcessor {
    pub fn new(ui_sender: Sender<UiMessage>) -> Self {
        Self {
            gain: SmoothValue::new(1.0, Linear::ease_in_out),
            meter: LevelMeter::new(Box::new(move |level: Level| {
                ui_sender.send(UiMessage::Level(level)).unwrap();
            })),
        }
    }
}

impl AudioProcessor for MyProcessor {
    type Message = MyMessage;

    fn prepare(&mut self, config: DeviceConfig) {
        self.gain.prepare(config.sample_rate, 100);
        self.meter
            .prepare(config.sample_rate, config.num_frames, 100);
        println!("Prepare is called with {:?}", config);
    }

    fn message_process(&mut self, message: MyMessage) {
        match message {
            MyMessage::Gain(gain) => self.gain.set_target_value(gain),
        }
    }

    fn process(
        &mut self,
        mut output: InterleavedAudioMut<'_, f32>,
        input: InterleavedAudio<'_, f32>,
    ) {
        if input.num_channels() > 0 {
            self.meter.process(input.channel_iter(0));
        }
        for (out_frame, in_frame) in output.frames_iter_mut().zip(input.frames_iter()) {
            let gain = self.gain.next_value();
            for (o, i) in out_frame.iter_mut().zip(in_frame.iter()) {
                *o = *i * gain;
            }
        }
    }
}
