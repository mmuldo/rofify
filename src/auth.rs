use rspotify::{
    prelude::*,
    scopes,
    AuthCodePkceSpotify,
    Credentials,
    OAuth
};

const CLIENT_ID: &str = "cb4b2d66eaa84bdc98e5e179a5bfc902";
const REDIRECT_URI: &str = "http://localhost:8888/callback";

const SCOPES: [&str; 17] = [
    "app-remote-control",
    "playlist-read-collaborative",
    "playlist-read-private",
    "playlist-modify-private",
    "playlist-modify-public",
    "streaming",
    "user-follow-read",
    "user-follow-modify",
    "user-library-modify",
    "user-library-read",
    "user-modify-playback-state",
    "user-read-currently-playing",
    "user-read-playback-state",
    "user-read-playback-position",
    "user-read-private",
    "user-read-recently-played",
    "user-top-read",
];

pub async fn get_token() -> AuthCodePkceSpotify{
    let creds = Credentials::new_pkce(CLIENT_ID);

    let oauth = OAuth {
        redirect_uri: REDIRECT_URI.to_string(),
        scopes: scopes!(&SCOPES.join(" ")),
        ..Default::default()
    };

    let mut spotify = AuthCodePkceSpotify::new(creds.clone(), oauth.clone());
    spotify.config.token_cached = true;

    let url = spotify.get_authorize_url(None).unwrap();
    spotify.prompt_for_token(&url).await.unwrap();
    
    spotify
}
