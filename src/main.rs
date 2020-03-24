use anyhow::Error;

fn main() {
    if let Err(e) = libav_decoder::run() {
        log_error(e.into())
    };
}

fn log_error(e: Error) {
    eprintln!("\n\x1b[31mERROR\x1b[0m: {}", e);
}
