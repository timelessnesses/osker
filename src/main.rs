use std::str::FromStr;

use poise;
use serenity;
use tlns_tetrio_calcs::ProfileStats;
use tokio;
use chrono;
use once_cell;
use serde_json;
use rayon::{self, iter::{IntoParallelRefIterator, ParallelIterator}};
use dashmap;

mod db;
mod types;
mod state;

pub static PLAYER_LIST: once_cell::sync::OnceCell<std::sync::Mutex<Vec<tlns_tetrio_calcs::ProfileStats>>> = once_cell::sync::OnceCell::with_value(std::sync::Mutex::new(Vec::new()));
pub static AVERAGE_PLAYERS: once_cell::sync::OnceCell<std::sync::Mutex<Vec<tlns_tetrio_calcs::ProfileStats>>> = once_cell::sync::OnceCell::with_value(std::sync::Mutex::new(Vec::new()));

struct PlayerSummarization {
    apm: f64,
    pps: f64,
    vs: f64,
    rank: tlns_tetrio_calcs::Ranks,
    count: u64,
    tr: f64,
    glicko: f64,
    rd: f64
}

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    let db = db::connect_to_db().await.expect("Failed to connect to database");
    let cloned = db.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::new(5, 0)).await;
            if let Err(_) = cloned.ping().await {
                panic!("Database disconnected!!");
            }
        }
    });

    let player_list = PLAYER_LIST.wait();
    let players = reqwest::get("https://ch.tetr.io/api/users/lists/league/all").await.expect("Failed to fetch players data").error_for_status().expect("Server respond non 200").json::<serde_json::Value>().await.expect("Failed to turn response to JSON")["data"]["users"];
    player_list.lock().expect("Failed to unlock the guard").append(&mut players.as_array().expect("Somehow the users array isn't an array").par_iter().map(|d| {
        tlns_tetrio_calcs::ProfileStats {
            apm: d["league"]["apm"].as_f64().unwrap() as f32,
            pps: d["league"]["pps"].as_f64().unwrap() as f32,
            vs: d["league"]["vs"].as_f64().unwrap() as f32,
            rank: Some(tlns_tetrio_calcs::Ranks::from_str(d["league"]["rank"].as_str().unwrap()).expect("Not expected rank")),
            tr: Some(d["league"]["rating"].as_f64().unwrap()),
            glicko: Some(d["league"]["glicko"].as_f64().unwrap()),
            name: Some(d["username"].as_str().unwrap().to_string()),
            pfp: Some(
                "https://tetr.io/user-content/avatars/".to_string()
                    + d["_id"].as_str().unwrap()
                    + ".jpg?rv="
                    + d["avatar_revision"].as_u64().unwrap().to_string().as_str(),
            ),
            rd: Some(d["league"]["rd"].as_f64().unwrap())
        }
    }).collect::<Vec<tlns_tetrio_calcs::ProfileStats>>());
    let stuffs: dashmap::DashMap<tlns_tetrio_calcs::Ranks, PlayerSummarization> = dashmap::DashMap::new();
    player_list.lock().expect("Failed to unlock the guard").par_iter().for_each(|v| {
        if stuffs.contains_key(&v.rank.unwrap_or(tlns_tetrio_calcs::Ranks::Z)) {
            stuffs.insert(v.rank.unwrap_or(tlns_tetrio_calcs::Ranks::Z), PlayerSummarization {
                apm: v.apm as f64,
                pps: v.pps as f64,
                vs: v.vs as f64,
                rank: v.rank.unwrap_or(tlns_tetrio_calcs::Ranks::Z),
                tr: v.tr.unwrap_or(0.0),
                glicko: v.glicko.unwrap_or(0.0),
                rd: v.rd.unwrap_or(0.0),
                count: 1
            });
        } else {
            stuffs.entry(v.rank.unwrap_or(tlns_tetrio_calcs::Ranks::Z)).and_modify(|a| {
                a.count += 1;
                a.apm += v.apm as f64;
                a.pps += v.pps as f64;
                a.vs += v.vs as f64;
                a.tr += v.tr.unwrap_or(0.0);
                a.glicko += v.glicko.unwrap_or(0.0);
                a.rd += v.rd.unwrap_or(0.0);
            });
        }
    });
    AVERAGE_PLAYERS.wait().lock().expect("Failed to unlock the guard").append(&mut stuffs.par_iter().map(|v| {
        let rank = v.rank.to_string();
        ProfileStats {
            apm: (v.apm / v.count as f64) as f32,
            pps: (v.pps / v.count as f64) as f32,
            vs: (v.vs / v.count as f64) as f32,
            rank: Some(v.rank),
            tr: Some(v.tr),
            name: Some(format!("$avg{rank}")),
            pfp: None,
            glicko: Some(v.glicko),
            rd: Some(v.rd)
        }
    }).collect::<Vec<tlns_tetrio_calcs::ProfileStats>>());
    let s = state::States {
        up_when: chrono::Local::now()
    };
    let bot = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            prefix_options: poise::PrefixFrameworkOptions { prefix: Some("osk".into()), mention_as_prefix: true, ..Default::default()},
            commands: vec![],
            event_handler: |ctx, event, framework, u| {
                std::boxed::Box::pin(events::listener(ctx, event, framework, u))
            },
            ..Default::default()
        })
        .setup(|ctx, ready, framework| {
            std::boxed::Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await.expect("Failed to register app commands");
                return Ok(s)
            })
        })
        .initialize_owners(true)
        .build();
    let client = serenity::Client::builder(std::env::var("OSKER_TOKEN").expect("OSKER_TOKEN"), serenity::model::gateway::GatewayIntents::all())
        .framework(bot)
        .status(serenity::model::user::OnlineStatus::Idle)
        .activity(serenity::gateway::ActivityData {
            name: "man these formula will fuck me up".to_string(),
            kind: serenity::model::gateway::ActivityType::Watching,
            state: None,
            url: None
        }).await.expect("Failed to login");
    client.start_autosharded().await.expect("Unexpected error, oh well.");
    log::info!("Bot started!");
}