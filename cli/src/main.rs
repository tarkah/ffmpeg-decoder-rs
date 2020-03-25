use anyhow::Error;
use env_logger::Env;
use log::{error, info};
use structopt::StructOpt;

use std::path::PathBuf;

fn main() {
    env_logger::from_env(Env::default().default_filter_or("info")).init();

    if let Err(e) = run() {
        log_error(e.into());
        std::process::exit(1);
    }
}

fn run() -> Result<(), Error> {
    let opts = Opts::from_args();

    let input = opts.input.display().to_string();

    let decoder = libav_decoder::Decoder::open(&input)?;

    let mut samples = vec![];
    for sample in decoder {
        samples.push(sample?);
    }

    let samples_u8 =
        unsafe { std::slice::from_raw_parts(samples.as_ptr() as *const u8, samples.len() * 2) };

    let mut out_path: PathBuf = opts.input;
    out_path.set_extension("raw");

    std::fs::write(&out_path, samples_u8)?;

    info!(
        "File successfully decoded, converted and saved to: {:?}",
        out_path
    );

    Ok(())
}

fn log_error(e: Error) {
    error!("{}", e);
}

#[derive(StructOpt)]
#[structopt(
    name = "libav-decoder-cli",
    about = "Convert input audio file sample format to signed 16bit little endian.

A `.raw` file will be saved with the same name alongside the input file."
)]
struct Opts {
    /// Input audio file
    #[structopt(parse(from_os_str))]
    input: PathBuf,
}
