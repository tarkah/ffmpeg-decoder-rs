use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("")]
    NoAudioStream,
    #[error("")]
    NullCodec,
    #[error("")]
    NullCodecContext,
    #[error("")]
    CodecParamsToContext,
    #[error("")]
    InitializeDecoder,
    #[error("")]
    NullFrame,
    #[error("")]
    ReadFrame(i32),
    #[error("")]
    SendPacket(i32),
    #[error("")]
    DraidDecoder(i32),
    #[error("")]
    ReceiveFrame(i32),
    #[error("Could not initialize SwrContext")]
    InitializeSwr,
}
