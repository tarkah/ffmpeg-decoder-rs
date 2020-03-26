mod decoder;
pub use decoder::Decoder;

mod error;
pub use error::Error;

#[cfg(feature = "rodio_source")]
mod rodio;
