use librespot_discovery::Credentials;
use std::env;
use tokio::io::AsyncBufReadExt;

use librespot_core::{
    cache::Cache, spotify_id::SpotifyItemType, Error, Session, SessionConfig, SpotifyId,
};
use librespot_playback::{
    audio_backend,
    config::{AudioFormat, PlayerConfig},
    mixer::NoOpVolume,
    player::Player,
};
use tokio::io::{self, BufReader};

#[tokio::main]
async fn main() {
    let mut builder = env_logger::Builder::new();
    builder.parse_filters("librespot=trace");
    builder.init();

    const DEFAULT_CACHE_DIR: &str = "./.cache";

    let cache = {
        let cache_dir = env::var("CACHE_PATH").unwrap_or(DEFAULT_CACHE_DIR.to_string());
        Cache::new(
            Some(cache_dir.clone()),
            Some(cache_dir.clone()),
            Some(cache_dir.clone()),
            None,
        )
        .expect("error reading from cache")
    };
    let credentials = cache.credentials().expect("error reading from credentials");

    let session_config = SessionConfig::default();
    let player_config = PlayerConfig::default();
    let audio_format = AudioFormat::default();
    let backend = audio_backend::find(Some("pipe".into())).unwrap();
    let mut session = connect_session(credentials.clone(), session_config.clone(), cache.clone())
        .await
        .expect("error connect session");
    let mut player = Player::new(
        player_config.clone(),
        session.clone(),
        Box::new(NoOpVolume),
        move || (backend)(None, audio_format),
    );

    let stdin = io::stdin();
    let mut lines = BufReader::new(stdin).lines();
    let mut command: Option<Command> = None;

    loop {
        tokio::select! {
            _ = async {}, if session.is_invalid() || player.is_invalid() => {
                match connect_session(credentials.clone(), session_config.clone(), cache.clone()).await {
                    Ok(new_session) => {
                        player =  Player::new(
                            player_config.clone(),
                            new_session.clone(),
                            Box::new(NoOpVolume),
                            move || (backend)(None, audio_format),
                        );
                        session = new_session;
                    },
                    Err(_) => {},
                }
            },
            line = async {
                match lines.next_line().await {
                    Ok(l) => l,
                    _ => None
                }
            }, if !session.is_invalid() && !player.is_invalid() && command.is_none() => {
                match line {
                    Some(l) => {
                        if let Ok(cmd) = Command::from_input(l) {
                            command = Some(cmd);
                        }
                    },
                    None => { },
                }
            },
            _ = async {}, if !session.is_invalid() && !player.is_invalid() && command.is_some() => {
                if let Some(cmd) = command {
                    match cmd {
                        Command::Play(track_id) => {
                            match SpotifyId::from_uri(&track_id) {
                                Ok(mut track) => {
                                    track.item_type = SpotifyItemType::Track;
                                    player.load(track, true, 0);
                                    player.await_end_of_track().await;
                                },
                                _ => {}
                            }
                        },
                        Command::Exit => {
                            break;
                        }
                    }
                }
                command = None;
            },
             _ = tokio::signal::ctrl_c() => {
                break;
            }
        }
    }
}

async fn connect_session(
    credentians: Credentials,
    session_config: SessionConfig,
    cache: Cache,
) -> Result<Session, Error> {
    let session = Session::new(session_config, Some(cache));
    session.connect(credentians, false).await?;
    Ok(session)
}

enum Command {
    Play(String),
    Exit,
}

impl Command {
    fn from_input(input: String) -> Result<Self, String> {
        let parts: Vec<&str> = input.trim().split_whitespace().collect();
        match parts.as_slice() {
            ["play", track_id] => Ok(Command::Play(track_id.to_string())),
            ["exit"] => Ok(Command::Exit),
            _ => Err("Invalid command".to_string()),
        }
    }
}
