pub struct AudioBlock<'a> {
    pub data: &'a [f32],
    pub num_channels: u16,
}

pub struct AudioBlockMut<'a> {
    pub data: &'a mut [f32],
    pub num_channels: u16,
}
