use std::{
    iter::{Skip, StepBy},
    slice::{Chunks, ChunksMut, Iter, IterMut},
};

pub struct InputBuffer<'a, T: Copy> {
    interleaved_audio: &'a [T],
    num_channels: usize,
    num_frames: usize,
}

impl<'a, T: Copy> InputBuffer<'a, T> {
    pub fn from(interleaved_audio: &'a [T], num_channels: usize) -> Self {
        Self {
            num_frames: interleaved_audio.len() / num_channels,
            interleaved_audio,
            num_channels,
        }
    }

    pub fn num_channels(&self) -> usize {
        self.num_channels
    }

    pub fn num_frames(&self) -> usize {
        self.num_frames
    }

    /// this will return an iterator over all contained frames
    /// a frame is a Chunk containing all channels
    /// a channel can be accessed by `chunk[num_channel]`
    pub fn frames_iter(&self) -> Chunks<'_, T> {
        self.interleaved_audio.chunks(self.num_channels)
    }

    /// this will return an iterator over one channel
    pub fn channel_iter(&self, channel: usize) -> StepBy<Skip<Iter<'_, T>>> {
        self.interleaved_audio
            .iter()
            .skip(channel)
            .step_by(self.num_channels)
    }

    pub fn interleaved_audio(&self) -> &[T] {
        self.interleaved_audio
    }

    /// returns the actual number of samples written
    pub fn copy_in_channel_buffer(&self, channel_buffer: &mut [T], channel: usize) -> usize {
        channel_buffer
            .iter_mut()
            .zip(self.channel_iter(channel))
            .for_each(|(o, i)| *o = *i);
        usize::min(channel_buffer.len(), self.num_frames)
    }
}

pub struct OutputBuffer<'a, T: Copy> {
    interleaved_audio: &'a mut [T],
    num_channels: usize,
    num_frames: usize,
}

impl<'a, T: Copy> OutputBuffer<'a, T> {
    pub fn from(interleaved_audio: &'a mut [T], num_channels: usize) -> Self {
        Self {
            num_frames: interleaved_audio.len() / num_channels,
            interleaved_audio,
            num_channels,
        }
    }

    pub fn num_channels(&self) -> usize {
        self.num_channels
    }

    pub fn num_frames(&self) -> usize {
        self.num_frames
    }

    /// this will return an iterator over all contained frames
    /// a frame is a Chunk containing all channels
    /// a channel can be accessed by `chunk[num_channel]`
    pub fn frames_iter(&self) -> Chunks<'_, T> {
        self.interleaved_audio.chunks(self.num_channels)
    }

    pub fn frames_iter_mut(&mut self) -> ChunksMut<'_, T> {
        self.interleaved_audio.chunks_mut(self.num_channels)
    }

    /// this will return an iterator over one channel
    pub fn channel_iter(&self, channel: usize) -> StepBy<Skip<Iter<'_, T>>> {
        self.interleaved_audio
            .iter()
            .skip(channel)
            .step_by(self.num_channels)
    }

    pub fn channel_iter_mut(&mut self, channel: usize) -> StepBy<Skip<IterMut<'_, T>>> {
        self.interleaved_audio
            .iter_mut()
            .skip(channel)
            .step_by(self.num_channels)
    }

    pub fn interleaved_audio(&self) -> &[T] {
        self.interleaved_audio
    }

    pub fn interleaved_audio_mut(&mut self) -> &mut [T] {
        self.interleaved_audio
    }

    /// returns the actual number of samples written
    pub fn copy_in_channel_buffer(&self, channel_buffer: &mut [T], channel: usize) -> usize {
        channel_buffer
            .iter_mut()
            .zip(self.channel_iter(channel))
            .for_each(|(o, i)| *o = *i);
        usize::min(channel_buffer.len(), self.num_frames)
    }

    // returns the actual number of samples written
    pub fn copy_from_channel_buffer(&mut self, channel_buffer: &[T], channel: usize) -> usize {
        self.channel_iter_mut(channel)
            .zip(channel_buffer.iter())
            .for_each(|(o, i)| *o = *i);
        usize::min(channel_buffer.len(), self.num_frames)
    }
}
