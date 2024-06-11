const COMMON_SAMPLE_RATES: &[u32] = &[44100, 48000, 88200, 96000, 192000];
const COMMON_FRAMES_PER_BUFFER: &[u32] = &[16, 32, 64, 128, 256, 512, 1024, 2048];
const DEFAULT_SAMPLE_RATE: u32 = 48000;
const DEFAULT_NUM_FRAMES: u32 = 512;

#[cfg(feature = "cpa")]
pub mod cpal_backend;
#[cfg(feature = "portaudio")]
pub mod portaudio_backend;
#[cfg(feature = "rtaud")]
pub mod rtaudio_backend;
