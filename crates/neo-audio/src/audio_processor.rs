use rt_tools::audio_buffers::{InputBuffer, OutputBuffer};

pub trait AudioProcessor<Message> {
    fn prepare(
        &mut self,
        sample_rate: u32,
        max_num_frames: u32,
        num_input_channels: u32,
        num_output_channels: u32,
    );

    fn message_process(&mut self, message: Message);

    fn process(&mut self, output: OutputBuffer<'_, f32>, input: InputBuffer<'_, f32>);
}
