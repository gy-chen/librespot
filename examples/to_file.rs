use std::env;

use librespot_core::authentication::Credentials;
use librespot_core::config::SessionConfig;
use librespot_core::session::{Session, SessionError};
use librespot_core::spotify_id::SpotifyId;
use librespot_playback::audio_backend;
use librespot_playback::config::{AudioFormat, PlayerConfig};
use librespot_playback::player::Player;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let player_config = PlayerConfig::default();
    let audio_format = AudioFormat::default();

    let args: Vec<_> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} TRACK_URI PATH", args[0]);
        return;
    }

    let track = SpotifyId::from_uri(&args[1]).unwrap();
    let backend = audio_backend::find(Some("gstreamer".into())).unwrap();
    let session = connect_session().await.unwrap();

    let (mut player, _) = Player::new(player_config, session, None, move || {
        backend(Some(format!("! audioconvert dithering=none ! wavenc ! decodebin ! audioconvert ! vorbisenc ! oggmux ! filesink location={}", args[2])), audio_format)
    });

    player.load(track, true, 0);
    player.await_end_of_track().await;
    player.stop();
}

async fn connect_session() -> Result<Session, SessionError> {
    let session_config = SessionConfig::default();
    let username = env::var("USERNAME").expect("expect USERNAME env variable");
    let password = env::var("PASSWORD").expect("expect PASSWORD env variable");
    let credentials = Credentials::with_password(username, password);
    Session::connect(session_config, credentials, None).await
}
