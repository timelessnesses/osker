use std::str::FromStr;

use rayon::{
    self,
    iter::{IntoParallelRefIterator, ParallelExtend, ParallelIterator},
};
use tlns_tetrio_calcs::ProfileStats;

mod commands;
// mod db;
mod state;
mod types;

mod errors;

#[derive(Debug)]
struct PlayerSummarization {
    apm: f64,
    pps: f64,
    vs: f64,
    rank: tlns_tetrio_calcs::Ranks,
    count: u64,
    tr: f64,
    glicko: f64,
    rd: f64,
}

fn setup_logging() {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{}] [{}] [{}] {}",
                chrono::Local::now().format("%Y/%m/%d][%H:%M:%S"),
                record.target(),
                record.level(),
                message
            ))
        })
        .level(log::LevelFilter::Debug)
        .level_for("tracing", log::LevelFilter::Off)
        .level_for("serenity::gateway::shard", log::LevelFilter::Warn)
        .level_for("serenity::gateway::ws", log::LevelFilter::Warn)
        .level_for("serenity::http::ratelimiting", log::LevelFilter::Warn)
        .level_for(
            "serenity::gateway::bridge::shard_manager",
            log::LevelFilter::Info,
        )
        .level_for("h2", log::LevelFilter::Off)
        .level_for("serenity::http::request", log::LevelFilter::Warn)
        .level_for("serenity::http::client", log::LevelFilter::Warn)
        .level_for("rustls", log::LevelFilter::Off)
        .level_for("hyper", log::LevelFilter::Warn)
        .level_for("hyper_util", log::LevelFilter::Warn)
        .level_for("tungstenite", log::LevelFilter::Off)
        .chain(std::io::stdout())
        .chain(fern::log_file("osker.log").unwrap())
        .apply()
        .unwrap();
}

async fn fetch_players_data() -> serde_json::Value {
    if cfg!(debug_assertions) {
        serde_json::from_str::<serde_json::Value>(include_str!("../info.txt"))
            .expect("Failed to parse")["data"]["users"]
            .clone()
    } else {
        reqwest::get("https://ch.tetr.io/api/users/lists/league/all")
            .await
            .expect("Failed to fetch players data")
            .error_for_status()
            .expect("Server responded with non-200 status")
            .json::<serde_json::Value>()
            .await
            .expect("Failed to parse response as JSON")["data"]["users"]
            .clone()
    }
}

async fn initialize_data(
    p: &std::sync::Arc<tokio::sync::Mutex<Vec<tlns_tetrio_calcs::ProfileStats>>>,
    a: &std::sync::Arc<tokio::sync::Mutex<Vec<tlns_tetrio_calcs::ProfileStats>>>,
) {
    log::info!("Reinitializing data");
    let mut locked = p.lock().await;
    let players = fetch_players_data().await;
    log::info!("Got new data from API");
    locked.clear();

    let stuffs: dashmap::DashMap<tlns_tetrio_calcs::Ranks, PlayerSummarization> =
        dashmap::DashMap::new();

    *locked = players
        .as_array()
        .expect("Users data is not an array")
        .par_iter()
        .map(|d| {
            let rank =
                tlns_tetrio_calcs::Ranks::from_str(d["league"]["rank"].as_str().unwrap_or("Z"))
                    .expect("Unexpected rank format");

            let profile_stat = tlns_tetrio_calcs::ProfileStats {
                apm: d["league"]["apm"].as_f64().unwrap_or(0.0) as f32,
                pps: d["league"]["pps"].as_f64().unwrap_or(0.0) as f32,
                vs: d["league"]["vs"].as_f64().unwrap_or(0.0) as f32,
                rank: Some(rank),
                tr: Some(d["league"]["rating"].as_f64().unwrap_or(0.0)),
                glicko: Some(d["league"]["glicko"].as_f64().unwrap_or(0.0)),
                name: Some(d["username"].as_str().unwrap_or("").to_string()),
                pfp: Some(format!(
                    "https://tetr.io/user-content/avatars/{}.jpg?rv={}",
                    d["_id"].as_str().unwrap_or("0"),
                    d["avatar_revision"].as_u64().unwrap_or(0)
                )),
                rd: Some(d["league"]["rd"].as_f64().unwrap_or(0.0)),
                is_real: true,
            };

            stuffs
                .entry(rank)
                .and_modify(|e| {
                    e.count += 1;
                    e.apm += profile_stat.apm as f64;
                    e.pps += profile_stat.pps as f64;
                    e.vs += profile_stat.vs as f64;
                    e.tr += profile_stat.tr.unwrap_or(0.0);
                    e.glicko += profile_stat.glicko.unwrap_or(0.0);
                    e.rd += profile_stat.rd.unwrap_or(0.0);
                })
                .or_insert_with(|| PlayerSummarization {
                    apm: profile_stat.apm as f64,
                    pps: profile_stat.pps as f64,
                    vs: profile_stat.vs as f64,
                    rank,
                    count: 1,
                    tr: profile_stat.tr.unwrap_or(0.0),
                    glicko: profile_stat.glicko.unwrap_or(0.0),
                    rd: profile_stat.rd.unwrap_or(0.0),
                });

            profile_stat
        })
        .collect();

    let mut locked2 = a.lock().await;
    locked2.clear();
    locked2.par_extend(stuffs.par_iter().map(|v| {
        let rank = v
            .key()
            .to_string()
            .to_uppercase()
            .replace("PLUS", "+")
            .replace("MINUS", "-");

        ProfileStats {
            apm: (v.value().apm / v.value().count as f64) as f32,
            pps: (v.value().pps / v.value().count as f64) as f32,
            vs: (v.value().vs / v.value().count as f64) as f32,
            rank: Some(v.value().rank),
            tr: Some(v.value().tr / v.value().count as f64),
            name: Some(format!("$avg{}", rank)),
            pfp: None,
            glicko: Some(v.value().glicko / v.value().count as f64),
            rd: Some(v.value().rd / v.value().count as f64),
            is_real: false,
        }
    }));
    log::info!("Done processing");
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    setup_logging();
    better_panic::Settings::new()
        .verbosity(better_panic::Verbosity::Full)
        .install();
    // let db = db::connect_to_db()
    //     .await
    //     .expect("Failed to connect to database");
    // let cloned = db.clone();

    let player_list = std::sync::Arc::new(tokio::sync::Mutex::new(Vec::new()));
    let average_players = std::sync::Arc::new(tokio::sync::Mutex::new(Vec::new()));
    initialize_data(&player_list, &average_players).await;

    let cloned_player_list = player_list.clone();
    let cloned_average_players = average_players.clone();
    // tokio::spawn(async move {
    //     loop {
    //         tokio::time::sleep(std::time::Duration::new(5, 0)).await;
    //         if cloned.ping().await.is_err() {
    //             panic!("Database disconnected!!");
    //         }
    //     }
    // });

    tokio::spawn(async move {
        loop {
            log::info!("Sleep for 5 minutes");
            tokio::time::sleep(std::time::Duration::new(300, 0)).await;
            initialize_data(&cloned_player_list, &cloned_average_players).await;
        }
    });

    let s = state::States {
        up_when: chrono::Local::now(),
        player_lists: player_list.clone(),
        avg_players: average_players.clone(),
    };
    let bot = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some("osk".into()),
                mention_as_prefix: true,
                ..Default::default()
            },
            commands: vec![commands::ts::ts(), commands::vs::vs()],
            ..Default::default()
        })
        .setup(move |ctx, _ready, framework| {
            std::boxed::Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands)
                    .await
                    .expect("Failed to register app commands");
                Ok(s)
            })
        })
        .initialize_owners(true)
        .build();
    let mut client = serenity::Client::builder(
        std::env::var("OSKER_TOKEN").expect("Token not found"),
        poise::serenity_prelude::GatewayIntents::all(),
    )
    .framework(bot)
    .await
    .expect("Error creating client");

    client.start().await.expect("Error starting client");
}
