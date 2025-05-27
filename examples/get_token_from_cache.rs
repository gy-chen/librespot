use std::env;
use std::path::PathBuf;

use librespot::core::{config::SessionConfig, session::Session};
use librespot_core::cache::Cache;

const SCOPES: &str =
    "streaming,user-read-playback-state,user-modify-playback-state,user-read-currently-playing";

#[tokio::main]
async fn main() {
    let mut builder = env_logger::Builder::new();
    builder.parse_filters("librespot=trace");
    builder.init();

    let session_config = SessionConfig::default();

    let args: Vec<_> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} PATH", args[0]);
        return;
    }

    let cred_dir: PathBuf = PathBuf::from(&args[1]);
    let cache = Cache::new(Some(cred_dir), None, None, None).unwrap();
    let credentials = cache.credentials().unwrap();
    
    println!("Connecting with token..");
    let session = Session::new(session_config.clone(), None);
    match session.connect(credentials, false).await {
        Ok(()) => println!("Session username: {:#?}", session.username()),
        Err(e) => {
            println!("Error connecting: {e}");
            return;
        }
    };

    let token = session.token_provider().get_token(SCOPES).await.unwrap();
    println!("Got me a token: {token:#?}");
}
