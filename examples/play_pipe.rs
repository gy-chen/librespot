use std::{env, path::PathBuf, process::exit};

use librespot::{
    core::{
        config::SessionConfig,
        session::Session,
        spotify_id::{SpotifyId, SpotifyItemType},
    },
    playback::{
        audio_backend,
        config::{AudioFormat, PlayerConfig},
        mixer::NoOpVolume,
        player::Player,
    },
};
use librespot_core::cache::Cache;

#[tokio::main]
async fn main() {
    let session_config = SessionConfig::default();
    let player_config = PlayerConfig::default();
    let audio_format = AudioFormat::default();

    let args: Vec<_> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} PATH TRACK", args[0]);
        return;
    }

    let credentials = {
        let cred_dir: PathBuf = PathBuf::from(&args[1]);
        let cache = Cache::new(Some(cred_dir), None, None, None).unwrap();
        cache.credentials().unwrap()
    };

    let mut track = SpotifyId::from_base62(&args[2]).unwrap();
    track.item_type = SpotifyItemType::Track;

    let backend = audio_backend::find(Some("pipe".into())).unwrap();

    let session = Session::new(session_config, None);
    if let Err(e) = session.connect(credentials, false).await {
        println!("Error connecting: {}", e);
        exit(1);
    }

    let player = Player::new(player_config, session, Box::new(NoOpVolume), move || {
        backend(None, audio_format)
    });

    player.load(track, true, 0);

    player.await_end_of_track().await;
}
