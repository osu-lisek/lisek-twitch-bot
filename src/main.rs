use std::{process::exit, sync::Arc, time::Duration};

use clap::Parser;
use config::RunConfiguration;
use regex::Regex;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use tokio::{sync::Mutex, time};
use tracing::{error, info, trace};
use twitch_irc::{
    login::StaticLoginCredentials, ClientConfig, SecureTCPTransport, TwitchIRCClient,
};

use crate::{bancho::bancho::send_message, db::{
    beatmap::Beatmap,
    integrations::{Integration, LinkedIndegration},
}};

mod config;
mod db;
mod bancho;

#[derive(Debug, Clone)]
struct Context {
    db: Pool<Postgres>,
}

async fn check_for_new_users(ctx: Arc<Pool<Postgres>>, users: Arc<Mutex<Vec<String>>>, check_time: Option<Duration>, client: Arc<TwitchIRCClient<SecureTCPTransport, StaticLoginCredentials>>) {
    let twitch_integration = Integration::find_by_slug(&ctx, String::from("twitch")).await;

    loop {
        let users_with_integrations = LinkedIndegration::fetch_users_with_integration(&ctx, twitch_integration.clone()).await;
        //Checking if there is some user disapeared from remaining users and if so, leaving the channel
        
        for user in users_with_integrations.clone() {
            let mut users_locked = users.lock().await;

            let id = &user.platform_id;
            if !users_locked.contains(&id) && user.visible {
                let user = user.clone();
                users_locked.push(id.to_string());
                let _ = client.join(user.display_name.to_lowercase());
                
            }

            if users_locked.contains(&id) && !user.visible {
                let user = user.clone();
                info!("Parting from {}", user.display_name);
                let _ = client.part(user.display_name);

                //Removing from users
                users_locked.retain(|x| x != id);
            }
        }

        time::sleep(check_time.unwrap_or(Duration::from_secs(5))).await;
    }
}

#[tokio::main]
async fn main() {
    let configuration = RunConfiguration::parse();

    tracing_subscriber::FmtSubscriber::builder()
        .with_level(true)
        .init();

    info!("Configuration loaded");

    let regexes = [Regex::new(
        r"https:\/\/lisek\.world\/(?:beatmaps|b|beatmapsets\/\d+)\/(?<beatmap_id>\d+)",
    ).expect("Failed to parse osu!lisek regex"),
    Regex::new(
        r"https://osu\.gatari\.pw\/b\/(?<beatmap_id>\d+)",
    )
    .expect("Failed to parse osu!gatari regex"),
    Regex::new(
        r"https://osu\.ppy\.sh\/beatmapsets/\d+#osu\/(?<beatmap_id>\d+)",
    )
    .expect("Failed to parse osu!bancho regex")
    ];

    info!("Regexes parsed");

    let db_pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&configuration.database_dsn)
        .await;

    if db_pool.is_err() {
        error!("Failed to connect to database");
        exit(1);
    }

    let db_pool = db_pool.unwrap();
    let context = Context { db: db_pool.clone() };

    let twitch_integration = Integration::find_by_slug(&context.db, String::from("twitch")).await;

    let users_with_twitch_integration =
        LinkedIndegration::fetch_users_with_integration(&context.db, twitch_integration).await;

    let joined_users = Arc::new(Mutex::new(Vec::new()));

    info!(
        "Found {} users with twitch integration.",
        users_with_twitch_integration.len()
    );

    let config = ClientConfig::new_simple(StaticLoginCredentials::new(
        configuration.twitch_username,
        Some(configuration.twitch_oauth_token),
    ));

    let (mut incoming_messages, client) =
        TwitchIRCClient::<SecureTCPTransport, StaticLoginCredentials>::new(config);

    let client_arc = Arc::new(client.clone());

    let db_arc = Arc::new(context.db.clone());

    let join_handle = tokio::spawn(async move {
        let ctx = client_arc.clone();
        tokio::spawn(check_for_new_users(db_arc.clone(), joined_users.clone(), None, ctx.clone()));
                    
        while let Some(message) = incoming_messages.recv().await {
            
            match message {
                twitch_irc::message::ServerMessage::Join(join_message) => {
                    info!("Joined {}", join_message.channel_login);
                }
                twitch_irc::message::ServerMessage::Privmsg(private_message) => {
                    trace!("Received message: {}", private_message.message_text);
                    for regex in regexes.clone().into_iter() {
                        let matches = regex.captures(&private_message.message_text);
                        if matches.is_none() {
                            continue;
                        }

                        let captures = matches.unwrap();
                        let id_capture = captures.name("beatmap_id").unwrap();
                        let beatmap_id = id_capture.as_str().parse::<i64>().unwrap();
                        trace!("Fetching beatmap with id: {}", beatmap_id);

                        let beatmap = Beatmap::fetch_from_db_by_id(&context.db, beatmap_id).await;

                        if let Some(beatmap) = beatmap {
                            trace!("Found beatmap: {:?}", beatmap);
                            ctx
                                .say(private_message.channel_login.to_owned().to_lowercase(), beatmap.format())
                                .await
                                .expect("Failed to send message.");

                            //Sending it to user
                            let user = users_with_twitch_integration.iter().find(|x| x.platform_id == private_message.channel_id);
                            if let Some(user) = user {
                                let response = send_message(user.user_id, beatmap.format_osu(private_message.sender.login.clone()), configuration.secret.clone()).await;

                                if let Some(response) = response {
                                    if !response.ok { 
                                        info!("Response: {:?}", response);
                                    }
                                }
                            }
                        }
                    }
                }
                twitch_irc::message::ServerMessage::Part(part_message) => {
                    info!("Parted {}", part_message.channel_login);
                }
                _ => {}
            }
        }
    });


    let _ = tokio::try_join!(
        tokio::spawn(async move {
            join_handle.await.unwrap();
        }),
    );
}
