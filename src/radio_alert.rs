use rodio::{Decoder, OutputStream, Sink, Source};
use std::fs::File;
use std::io::BufReader;

pub fn play_sound() {
    let (_stream, handle) = rodio::OutputStream::try_default().unwrap();
    let sink = rodio::Sink::try_new(&handle).unwrap();

    let file = File::open("src/assets/alert.wav").unwrap();
    sink.append(rodio::Decoder::new(BufReader::new(file)).unwrap());
    sink.sleep_until_end();
}

#[cfg(test)]
mod tests {
    use super::*;
    use rodio::Source;

    #[test]
    fn test_play_sound_v0() {
        play_sound();
    }

    #[test]
    fn test_play_sound() {
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        let file = File::open("src/assets/alert.wav").unwrap();
        let source = Decoder::new(BufReader::new(file)).unwrap();
        stream_handle.play_raw(source.convert_samples());
        std::thread::sleep(std::time::Duration::from_secs(5));
    }
}
