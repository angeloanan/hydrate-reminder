use rodio::Decoder;

const NOTIFICATION_AUDIO: &[u8] = include_bytes!("../assets/notif.mp3");
const DRINK_AUDIO: &[u8] = include_bytes!("../assets/gulp.mp3");

pub fn notification_audio() -> Decoder<std::io::Cursor<&'static [u8]>> {
    let audio_buffer = std::io::Cursor::new(NOTIFICATION_AUDIO);
    Decoder::new_mp3(audio_buffer).unwrap()
}

pub fn drink_audio() -> Decoder<std::io::Cursor<&'static [u8]>> {
    let audio_buffer = std::io::Cursor::new(DRINK_AUDIO);
    Decoder::new_mp3(audio_buffer).unwrap()
}
