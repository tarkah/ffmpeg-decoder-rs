use crate::Decoder;

use rodio::source::Source;

use std::time::Duration;

impl Source for Decoder {
    #[inline]
    fn current_frame_len(&self) -> Option<usize> {
        //self.current_frame_len()
        None
    }

    #[inline]
    fn channels(&self) -> u16 {
        self._channels()
    }

    #[inline]
    fn sample_rate(&self) -> u32 {
        self._sample_rate()
    }

    #[inline]
    fn total_duration(&self) -> Option<Duration> {
        self._total_duration()
    }
}
