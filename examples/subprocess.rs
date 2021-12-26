use std::env;

use tokio::io::{AsyncBufReadExt, BufReader, stdin};

use librespot::core::config::SessionConfig;
use librespot_core::authentication::Credentials;
use librespot_core::session::Session;
use librespot_core::spotify_id::SpotifyId;
use librespot_playback::audio_backend;
use librespot_playback::config::{AudioFormat, PlayerConfig};
use librespot_playback::player::Player;

#[tokio::main]
async fn main() {
    let session_config = SessionConfig::default();
    let player_config = PlayerConfig::default();
    let audio_format = AudioFormat::default();

    let username = env::var("USERNAME").expect("expect USERNAME env variable");
    let password = env::var("PASSWORD").expect("expect PASSWORD env variable");

    let credentials = Credentials::with_password(username, password);

    let backend = audio_backend::find(Some("pipe".into())).unwrap();

    let session = Session::connect(session_config, credentials, None)
        .await
        .unwrap();

    let (mut player, _) = Player::new(player_config, session, None, move || {
        backend(None, audio_format)
    });

    let stdin = stdin();
    let mut stdin = BufReader::new(stdin);

    loop {
        let mut command = String::new();

        stdin.read_line(&mut command)
            .await
            .unwrap();

        let command = Command::parse(&command);

        match command {
            Ok(command) => {
                match execute_command(&mut player, &command).await {
                    Ok(_) => eprintln!("="),
                    Err(msg) => eprintln!("?{}", msg)
                }
            }
            Err(e) => eprintln!("?{}", e)
        }
    }
}

async fn execute_command(player: &mut Player, command: &Command) -> Result<(), String> {
    match command.type_ {
        CommandType::PLAY => _play(player, command.track_id).await
    }
}

async fn _play(player: &mut Player, track_id: SpotifyId) -> Result<(), String> {
    player.load(track_id, true, 0);
    player.await_end_of_track().await;
    return Ok(());
}

enum CommandType {
    PLAY
}

struct Command {
    type_: CommandType,
    track_id: SpotifyId,
}

impl Command {
    fn parse(raw: &str) -> Result<Command, &str> {
        let args: Vec<&str> = raw.split_whitespace().collect();
        match args.get(0) {
            Some(&"play") => match args.get(1) {
                Some(&track_id) => {
                    match SpotifyId::from_uri(&track_id) {
                        Ok(track_id) => Ok(
                            Command {
                                type_: CommandType::PLAY,
                                track_id,
                            }
                        ),
                        Err(_) => match SpotifyId::from_base62(&track_id) {
                            Ok(track_id) => Ok(
                                Command {
                                    type_: CommandType::PLAY,
                                    track_id,
                                }
                            ),
                            Err(_) => Err("cannot parse track_id:")
                        }
                    }
                }
                _ => Err("expect track_id parameter")
            },
            _ => Err("unknown command")
        }
    }
}
