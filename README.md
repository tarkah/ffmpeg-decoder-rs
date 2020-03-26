# lib-av-decoder

Decodes input audio files and converts sample format to signed 16bit little endian

## CLI


### Save decoded, converted data to file
```
cargo run --release -- save assets/BGM_AI.at3
```

### Play with rodio
```
cargo run --release -- play assets/BGM_AI.at3
```