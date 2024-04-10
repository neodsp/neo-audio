use iced::{
    widget::{column, combo_box, container, scrollable},
    Element, Length,
};
use neo_audio::{prelude::*, processors::player::Sender};
use rt_tools::{
    level_meter::{Level, LevelMeter},
    smooth_value::{Easing, Linear, SmoothValue},
};

fn main() -> iced::Result {
    iced::run(
        "neo-audio Iced Demo",
        NeoAudioIcedApp::update,
        NeoAudioIcedApp::view,
    )
}

#[derive(Debug, Clone)]
enum AppMessage {
    ApiChanged(String),
    OutputDeviceChanged(String),
    NumOutputChChanged(u32),
    InputDeviceChanged(String),
    NumInputChChanged(u32),
    SampleRateChanged(u32),
    NumFramesChanged(u32),
}

struct NeoAudioIcedApp {
    neo_audio: NeoAudio<RtAudioBackend, MyProcessor>,
    apis: combo_box::State<String>,
    output_devices: combo_box::State<String>,
    output_channels: combo_box::State<u32>,
    input_devices: combo_box::State<String>,
    input_channels: combo_box::State<u32>,
    sample_rates: combo_box::State<u32>,
    num_frames: combo_box::State<u32>,
    selected_config: DeviceConfig,
}

impl NeoAudioIcedApp {
    fn new() -> Self {
        let neo_audio = NeoAudio::<RtAudioBackend, MyProcessor>::new().unwrap();
        Self {
            apis: combo_box::State::new(neo_audio.backend().available_apis()),
            output_devices: combo_box::State::new(neo_audio.backend().available_output_devices()),
            output_channels: combo_box::State::new(
                (1..neo_audio.backend().available_num_output_channels() + 1).collect(),
            ),
            input_devices: combo_box::State::new(neo_audio.backend().available_input_devices()),
            input_channels: combo_box::State::new(
                (1..neo_audio.backend().available_num_input_channels() + 1).collect(),
            ),
            sample_rates: combo_box::State::new(neo_audio.backend().available_sample_rates()),
            num_frames: combo_box::State::new(neo_audio.backend().available_num_frames()),
            selected_config: neo_audio.backend().config(),
            neo_audio,
        }
    }

    fn update_devices(&mut self) {
        self.neo_audio.backend_mut().update_devices().unwrap();
        self.apis = combo_box::State::new(self.neo_audio.backend().available_apis());
        self.output_devices =
            combo_box::State::new(self.neo_audio.backend().available_output_devices());
        self.output_channels = combo_box::State::new(
            (1..self.neo_audio.backend().available_num_output_channels() + 1).collect(),
        );
        self.input_devices =
            combo_box::State::new(self.neo_audio.backend().available_input_devices());
        self.input_channels = combo_box::State::new(
            (1..self.neo_audio.backend().available_num_input_channels() + 1).collect(),
        );
        self.sample_rates =
            combo_box::State::new(self.neo_audio.backend().available_sample_rates());
        self.num_frames = combo_box::State::new(self.neo_audio.backend().available_num_frames());
        self.selected_config = self.neo_audio.backend().config();
    }

    fn update(&mut self, message: AppMessage) {
        match message {
            AppMessage::ApiChanged(api) => {
                self.neo_audio.backend_mut().set_api(&api).unwrap();
            }
            AppMessage::OutputDeviceChanged(device) => {
                self.neo_audio
                    .backend_mut()
                    .set_output_device(DeviceName::Name(device))
                    .unwrap();
            }
            AppMessage::NumOutputChChanged(ch) => {
                self.neo_audio
                    .backend_mut()
                    .set_num_output_channels(ch)
                    .unwrap();
            }
            AppMessage::InputDeviceChanged(device) => {
                self.neo_audio
                    .backend_mut()
                    .set_input_device(DeviceName::Name(device))
                    .unwrap();
            }
            AppMessage::NumInputChChanged(ch) => {
                self.neo_audio
                    .backend_mut()
                    .set_num_input_channels(ch)
                    .unwrap();
            }
            AppMessage::SampleRateChanged(sr) => {
                self.neo_audio.backend_mut().set_sample_rate(sr).unwrap();
            }
            AppMessage::NumFramesChanged(nf) => {
                self.neo_audio.backend_mut().set_num_frames(nf).unwrap();
            }
        };
        self.update_devices();
    }

    fn view(&self) -> Element<AppMessage> {
        let api_combo_box = combo_box(
            &self.apis,
            &self.selected_config.api,
            Some(&self.selected_config.api),
            AppMessage::ApiChanged,
        )
        .width(250);

        let device: Option<String> = self.selected_config.output_device.clone().into();
        let output_combo_box = combo_box(
            &self.output_devices,
            "",
            device.as_ref(),
            AppMessage::OutputDeviceChanged,
        )
        .width(250);

        let output_ch_combo_box = combo_box(
            &self.output_channels,
            &self.selected_config.num_output_ch.to_string(),
            Some(&self.selected_config.num_output_ch),
            AppMessage::NumOutputChChanged,
        )
        .width(250);

        let device: Option<String> = self.selected_config.input_device.clone().into();
        let input_combo_box = combo_box(
            &self.input_devices,
            "",
            device.as_ref(),
            AppMessage::InputDeviceChanged,
        )
        .width(250);

        let input_ch_combo_box = combo_box(
            &self.input_channels,
            &self.selected_config.num_input_ch.to_string(),
            Some(&self.selected_config.num_input_ch),
            AppMessage::NumInputChChanged,
        )
        .width(250);

        let sample_rates_combo_box = combo_box(
            &self.sample_rates,
            &self.selected_config.sample_rate.to_string(),
            Some(&self.selected_config.sample_rate),
            AppMessage::SampleRateChanged,
        )
        .width(250);

        let num_frames_combo_box = combo_box(
            &self.num_frames,
            &self.selected_config.num_frames.to_string(),
            Some(&self.selected_config.num_frames),
            AppMessage::NumFramesChanged,
        )
        .width(250);

        let content = column![
            "Api",
            api_combo_box,
            "Output Device",
            output_combo_box,
            "Num Output Channels",
            output_ch_combo_box,
            "Input Device",
            input_combo_box,
            "Num Input Channels",
            input_ch_combo_box,
            "Sample Rate",
            sample_rates_combo_box,
            "Num Frames",
            num_frames_combo_box,
        ]
        .width(Length::Fill)
        .align_items(iced::Alignment::Center)
        .spacing(10);

        container(scrollable(content))
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}

impl Default for NeoAudioIcedApp {
    fn default() -> Self {
        Self::new()
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
