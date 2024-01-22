use clap::{Parser};


#[derive(Parser, Debug)]
pub struct RunConfiguration {
    #[arg(short, long, env)]
    pub database_dsn: String,

    #[arg(long, env)]
    pub twitch_username: String,
    
    #[arg(long, env)]
    pub twitch_oauth_token: String,

    #[arg(long, env)]
    pub secret: String,
}