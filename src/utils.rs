use notify_rust::{Notification, Timeout};
use rodio::{source::Source, Decoder, OutputStream};
use std::fs::File;
use std::io::BufReader;

use crate::enums::NotificationSound;

pub fn push_notification(body: &str) {
    Notification::new()
        .summary("Discord Watcher")
        .timeout(Timeout::Milliseconds(500))
        .auto_icon()
        .sound_name(&NotificationSound::Reminder.to_str())
        .body(body)
        .finalize()
        .show()
        .unwrap();
}

pub fn play_sound() {
    // Get a output stream handle to the default physical sound device
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    // Load a sound from a file, using a path relative to Cargo.toml
    let file = BufReader::new(File::open("assets/notif_sound.wav").unwrap());
    // Decode that sound file into a source
    let source = Decoder::new_wav(file).unwrap();
    // Play the sound directly on the device
    match stream_handle.play_raw(source.convert_samples()) {
        Ok(_) => (),
        Err(e) => eprintln!("Error when playing sound: {}", e),
    }
    // The sound plays in a separate audio thread,
    // so we need to keep the main thread alive while it's playing.
    std::thread::sleep(std::time::Duration::from_secs(1));
}
