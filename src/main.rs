use std::str::FromStr;

use rayon::{
    self,
    iter::{IntoParallelRefIterator, ParallelExtend, ParallelIterator},
};
use tlns_tetrio_calcs::{ProfileStats, Ranks};

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

async fn fetch_players_data() -> Vec<ProfileStats> {
    let session_id = "lalalalalalalala";
    const LIMIT: u64 = 100;
    let mut prisecter: Option<String> = None;
    let client = reqwest::ClientBuilder::new()
        .user_agent("tlns-tetrio-calcs")
        .build()
        .unwrap();
    let mut should_break = false;
    let mut output = Vec::new();
    let mut count_stream = 0;
    while !should_break && count_stream == 50 { // im trying to be kind osk
        count_stream += 1;
        log::debug!("Request number {count_stream}");
        let mut res = client
            .get(tlns_tetrio_calcs::API.to_string() + "users/by/league")
            .query(&[("limit", LIMIT.to_string())]);
        if let Some(a) = &prisecter {
            res = res.query(&[("after", &a.as_str())]);
            log::info!("used prisecter")
        }
        res = res.header("X-Session-ID", session_id);
        log::info!("used session id");
        let response = res
            .send()
            .await
            .expect("Failed to request players data")
            .error_for_status()
            .expect("Probably Ratelimited?");
        log::info!("{:#?}", response);
        let response = response.json::<serde_json::Value>().await.unwrap();
        if !response["success"].as_bool().unwrap() {
            panic!("Failed to fetch data")
        }
        should_break = response["data"]["entries"].as_array().unwrap().len() != 100;
        for data in response["data"]["entries"].as_array().unwrap() {
            output.push(ProfileStats {
                apm: data["league"]["apm"].as_f64().unwrap() as f32,
                pps: data["league"]["pps"].as_f64().unwrap() as f32,
                vs: data["league"]["vs"].as_f64().unwrap() as f32,
                rank: Ranks::from_str(data["league"]["rank"].as_str().unwrap()).ok(),
                tr: data["league"]["tr"].as_f64(),
                name: Some(data["username"].as_str().unwrap().to_string()),
                pfp: Some(
                    "https://tetr.io/user-content/avatars/".to_string()
                        + data["_id"].as_str().unwrap()
                        + ".jpg",
                ),
                glicko: data["league"]["glicko"].as_f64(),
                rd: data["league"]["rd"].as_f64(),
                is_real: true,
            });
        }
        let p = response["data"]["entries"]
            .as_array()
            .unwrap()
            .last()
            .unwrap()["p"]
            .clone();
        prisecter = Some(format!(
            "{}:{}:{}",
            p["pri"].as_f64().unwrap(),
            p["sec"].as_f64().unwrap(),
            p["ter"].as_f64().unwrap()
        ));
    }
    println!("{:#?}", output);
    output
}

async fn initialize_data(
    p: &std::sync::Arc<tokio::sync::RwLock<Vec<tlns_tetrio_calcs::ProfileStats>>>,
    a: &std::sync::Arc<tokio::sync::RwLock<Vec<tlns_tetrio_calcs::ProfileStats>>>,
) {
    log::info!("Reinitializing data");
    let mut locked = p.write().await;
    let players = fetch_players_data().await;
    log::info!("Got new data from API");
    locked.clear();

    let stuffs: dashmap::DashMap<tlns_tetrio_calcs::Ranks, PlayerSummarization> =
        dashmap::DashMap::new();
    let count: dashmap::DashMap<tlns_tetrio_calcs::Ranks, u128> = dashmap::DashMap::new();
    players.par_iter().for_each(|d| {
        stuffs
            .entry(d.rank.unwrap())
            .and_modify(|e| {
                e.count += 1;
                e.apm += d.apm as f64;
                e.pps += d.pps as f64;
                e.vs += d.vs as f64;
                e.tr += d.tr.unwrap_or(0.0);
                e.glicko += d.glicko.unwrap_or(0.0);
                e.rd += d.rd.unwrap_or(0.0);
            })
            .or_insert_with(|| PlayerSummarization {
                apm: d.apm as f64,
                pps: d.pps as f64,
                vs: d.vs as f64,
                rank: d.rank.unwrap(),
                count: 1,
                tr: d.tr.unwrap_or(0.0),
                glicko: d.glicko.unwrap_or(0.0),
                rd: d.rd.unwrap_or(0.0),
            });
        count
            .entry(d.rank.unwrap())
            .and_modify(|i| *i += 1u128)
            .or_insert(1u128);
    });
    *locked = players;

    let mut locked2 = a.write().await;
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
    let mut apm = 0.0;
    let mut pps = 0.0;
    let mut vs = 0.0;
    let mut glicko = 0.0;
    let mut tr = 0.0;
    let mut rd = 0.0;

    for x in stuffs.iter() {
        apm += x.apm;
        pps += x.pps;
        vs += x.vs;
        glicko += x.glicko;
        tr += x.tr;
        rd += x.rd;
    }
    let c = count.len() as f64;
    let mut max_rank = Ranks::Z;
    let mut max_count = 0;
    for rank in count {
        if max_count < rank.1 {
            max_rank = rank.0;
            max_count = rank.1;
        }
    }
    let averaged_player_base = ProfileStats {
        apm: (apm / c) as f32,
        pps: (pps / c) as f32,
        vs: (vs / c) as f32,
        rank: Some(max_rank),
        tr: Some(tr / c),
        name: Some("$avgALL".to_string()),
        pfp: None,
        glicko: Some(glicko / c),
        rd: Some(rd / c),
        is_real: false,
    };
    log::debug!("{:#?}", averaged_player_base);
    locked2.push(averaged_player_base);
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

    let player_list = std::sync::Arc::new(tokio::sync::RwLock::new(Vec::new()));
    let average_players = std::sync::Arc::new(tokio::sync::RwLock::new(Vec::new()));
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
            commands: vec![
                commands::ts::ts(),
                commands::vs::vs(),
                commands::ping::ping(),
            ],
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

    client
        .start_autosharded()
        .await
        .expect("Error starting client");
}
