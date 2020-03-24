#![allow(improper_ctypes)]

pub mod avcodec;
pub mod avformat;
pub mod avutil;

#[cfg(test)]
mod tests {
    use super::avcodec::avcodec_configuration;
    use super::avformat::avformat_configuration;
    use super::avutil::avutil_configuration;
    use std::ffi::CStr;
    #[test]
    fn version() {
        unsafe {
            println!(
                "{}{}{}",
                CStr::from_ptr(avcodec_configuration()).to_string_lossy(),
                CStr::from_ptr(avformat_configuration()).to_string_lossy(),
                CStr::from_ptr(avutil_configuration()).to_string_lossy()
            )
        };
    }
}
