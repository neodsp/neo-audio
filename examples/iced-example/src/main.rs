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
    InputDeviceChanged(String),
}

struct NeoAudioIcedApp {
    neo_audio: NeoAudio<RtAudioBackend, MyProcessor>,
    apis: combo_box::State<String>,
    output_devices: combo_box::State<String>,
    input_devices: combo_box::State<String>,
    selected_config: DeviceConfig,
}

impl NeoAudioIcedApp {
    fn new() -> Self {
        let neo_audio = NeoAudio::<RtAudioBackend, MyProcessor>::new().unwrap();
        Self {
            apis: combo_box::State::new(neo_audio.backend().available_apis()),
            output_devices: combo_box::State::new(neo_audio.backend().available_output_devices()),
            input_devices: combo_box::State::new(neo_audio.backend().available_input_devices()),
            selected_config: neo_audio.backend().config(),
            neo_audio,
        }
    }

    fn update(&mut self, message: AppMessage) {
        match message {
            AppMessage::ApiChanged(api) => dbg!(api),
            AppMessage::OutputDeviceChanged(_) => todo!(),
            AppMessage::InputDeviceChanged(_) => todo!(),
        };
    }

    fn view(&self) -> Element<AppMessage> {
        let api_combo_box = combo_box(
            &self.apis,
            &self.selected_config.api,
            Some(&self.selected_config.api),
            AppMessage::ApiChanged,
        )
        .width(250);

        let output_device: Option<String> = self.selected_config.output_device.clone().into();
        let output_combo_box = combo_box(
            &self.output_devices,
            match &output_device {
                Some(name) => name,
                None => "",
            },
            output_device.as_ref(),
            AppMessage::OutputDeviceChanged,
        )
        .width(250);

        // let input_device: Option<String> = self.selected_config.input_device.clone().into();
        // let input_combo_box = combo_box(
        //     &self.input_devices,
        //     match &input_device {
        //         Some(name) => name,
        //         None => "",
        //     },
        //     input_device.as_ref(),
        //     AppMessage::InputDeviceChanged,
        // )
        // .width(250);

        let content = column![
            "Api",
            api_combo_box,
            "Output Device",
            output_combo_box,
            // "Input Device",
            // input_device
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
