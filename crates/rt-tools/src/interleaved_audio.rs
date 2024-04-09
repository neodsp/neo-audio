use std::{
    iter::{Skip, StepBy},
    slice::{Chunks, ChunksMut, Iter, IterMut},
};

#[derive(Clone)]
pub struct InterleavedAudio<'a, T: Copy> {
    data: &'a [T],
    num_channels: usize,
    num_frames: usize,
}

impl<'a, T: Copy> InterleavedAudio<'a, T> {
    pub fn from_slice(data: &'a [T], num_channels: usize) -> Self {
        Self {
            num_frames: data.len() / num_channels,
            data,
            num_channels,
        }
    }

    pub fn from_audio_data_mut(&mut self, audio_data: InterleavedAudioMut<'a, T>) -> Self {
        Self {
            num_frames: audio_data.num_frames(),
            data: audio_data.data,
            num_channels: audio_data.num_channels,
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
        self.data.chunks(self.num_channels)
    }

    /// this will return an iterator over one channel
    pub fn channel_iter(&self, channel: usize) -> impl Iterator<Item = &'a T> {
        self.data.iter().skip(channel).step_by(self.num_channels)
    }

    pub fn data(&self) -> &[T] {
        self.data
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

pub struct InterleavedAudioMut<'a, T: Copy> {
    data: &'a mut [T],
    num_channels: usize,
    num_frames: usize,
}

impl<'a, T: Copy> InterleavedAudioMut<'a, T> {
    pub fn from_slice(data: &'a mut [T], num_channels: usize) -> Self {
        Self {
            num_frames: data.len() / num_channels,
            data,
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
        self.data.chunks(self.num_channels)
    }

    pub fn frames_iter_mut(&mut self) -> ChunksMut<'_, T> {
        self.data.chunks_mut(self.num_channels)
    }

    /// this will return an iterator over one channel
    pub fn channel_iter(&self, channel: usize) -> StepBy<Skip<Iter<'_, T>>> {
        self.data.iter().skip(channel).step_by(self.num_channels)
    }

    pub fn channel_iter_mut(&mut self, channel: usize) -> StepBy<Skip<IterMut<'_, T>>> {
        self.data
            .iter_mut()
            .skip(channel)
            .step_by(self.num_channels)
    }

    pub fn data(&self) -> &[T] {
        self.data
    }

    pub fn data_mut(&mut self) -> &mut [T] {
        self.data
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
