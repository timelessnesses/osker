use std::str::FromStr;

use poise;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use regex;

const FUNNY_IMAGE: &'static str = "https://statics.timelessnesses.me/poiuu_drawings/sd.png";
const DETECT_AVG_PATTERN: &'static str = r"^\$avg([a-zA-Z])$";
pub const ZERO_WIDTH_SPACE: &'static str = "\u{200b}";

#[poise::command(prefix_command, slash_command)]
pub async fn ts(
    ctx: crate::types::Context<'_>,
    args: Vec<String>,
) -> Result<(), crate::types::Error> {
    ctx.defer().await?;

    let mut players_list = ctx.data().player_lists.lock().await;
    let average_players = ctx.data().avg_players.lock().await;
    let mut should_add = false;
    let player: tlns_tetrio_calcs::ProfileStats;

    if args.len() == 3 {
        player = tlns_tetrio_calcs::ProfileStats::from_stat(
            args[0].parse()?,
            args[1].parse()?,
            args[2].parse()?,
        );
    } else {
        let r = regex::Regex::new(&DETECT_AVG_PATTERN)?;
        if let Some(avg_rank) = r.captures(&args[0]) {
            let rank = avg_rank
                .get(1)
                .ok_or(crate::errors::Errors::RankNotFoundError)?;
            player = if let Some(avg) = average_players.iter().find(|i| {
                i.rank.unwrap()
                    == tlns_tetrio_calcs::Ranks::from_str(&rank.as_str().to_lowercase()).unwrap()
            }) {
                avg.clone()
            } else {
                return Err(crate::errors::Errors::RankNotFoundError.into());
            };
        } else {
            if let Some(p) = players_list
                .par_iter()
                .find_first(|i| i.name.clone().unwrap_or_else(|| "".to_string()) == args[0])
            {
                player = p.clone();
            } else {
                player = tlns_tetrio_calcs::ProfileStats::from_username(&args[0]).await?;
                should_add = true;
            }
        }
    }

    if should_add {
        players_list.push(player.clone());
    }

    let embed = build_player_embed(&player);
    ctx.send(poise::CreateReply::default().embed(embed).reply(true))
        .await?;
    Ok(())
}

fn build_player_embed(
    player: &tlns_tetrio_calcs::ProfileStats,
) -> poise::serenity_prelude::CreateEmbed {
    poise::serenity_prelude::CreateEmbed::new()
        .colour(poise::serenity_prelude::Color::from_rgb(0, 153, 255))
        .title(match player.is_real {
            true => player.name.as_ref().unwrap().to_string(),
            false => {
                format!(
                    "ADVANCED STATS OF [{}, {}, {}]",
                    player.apm, player.pps, player.vs
                )
            }
        })
        .url(match player.is_real {
            true => "https://ch.tetr.io/u/".to_string() + &player.name.as_ref().unwrap(),
            false => "https://ch.tetr.io".to_string(),
        })
        .author(
            poise::serenity_prelude::CreateEmbedAuthor::new("timelessnesses")
                .icon_url(FUNNY_IMAGE)
                .url("https://github.com/timelessnesses/osker"),
        )
        .description("osker - A sheetBot rewrites in Rust that fetches advanced statistics from ch.tetr.io API")
        .field("APM", tlns_tetrio_calcs::truncate(player.apm as f64, 2).to_string(), true)
        .field("PPS", tlns_tetrio_calcs::truncate(player.pps as f64, 2).to_string(), true)
        .field("VS", tlns_tetrio_calcs::truncate(player.vs as f64, 2).to_string(), true)
        .field("DS/Piece", tlns_tetrio_calcs::truncate(player.ds_pieces(), 4).to_string(), true)
        .field("APP", tlns_tetrio_calcs::truncate(player.app(), 4).to_string(), true)
        .field("APP+DS/Piece", tlns_tetrio_calcs::truncate(player.app_ds_per_pieces(), 4).to_string(), true)
        .field(ZERO_WIDTH_SPACE, ZERO_WIDTH_SPACE, true)
        .field("Rank", player.rank.unwrap_or(tlns_tetrio_calcs::Ranks::Z).to_string(), true)
        .field(ZERO_WIDTH_SPACE, ZERO_WIDTH_SPACE, true)
        .field("Advanced:",
            "➤DS/Second: **".to_string() + &tlns_tetrio_calcs::truncate(player.ds_seconds(), 4).to_string() + "**\n" +
            "➤VS/APM: **" + &tlns_tetrio_calcs::truncate(player.vs_apm(), 4).to_string() + "**\n" +
            "➤Garbage Efficiency: **" + &tlns_tetrio_calcs::truncate(player.garbage_efficiency(), 4).to_string() + "**\n" +
            "➤Cheese Index: **" + &tlns_tetrio_calcs::truncate(player.cheese_index(), 4).to_string() + "**\n" +
            "➤Weighted APP: **" + &tlns_tetrio_calcs::truncate(player.weighted_app(), 4).to_string() + "**\n\n" 
        , true)
        .field("Ranking:", match player.is_real {
            true => {
                "➤Area: **".to_string() + &tlns_tetrio_calcs::truncate(player.area(), 4).to_string() + "**\n" +
                "➤TR: **" + &tlns_tetrio_calcs::truncate(player.tr.unwrap(), 2).to_string() + "**\n" +
                "➤Estimated TR: **" + &tlns_tetrio_calcs::truncate(player.estimated_tr(), 2).to_string() + "**\n" +
                "➤Estimated TR Accuracy: **" + &tlns_tetrio_calcs::truncate(player.accuracy_tr(), 2).to_string() + "**\n" +
                "➤Glicko±RD: **" + &tlns_tetrio_calcs::truncate(player.glicko.unwrap(), 2).to_string() + "**±" + &tlns_tetrio_calcs::truncate(player.rd.unwrap(), 2).to_string() + "\n\n"
            },
            false => {
                "This is a dummy user, unable to process TRs".to_string()
            }
        }, true)
        .field("Playstyle:", 
        "➤Opener: **".to_string() + &tlns_tetrio_calcs::truncate(player.opener(), 4).to_string() + "**\n" +
        "➤Plonk: **" + &tlns_tetrio_calcs::truncate(player.plonk(), 4).to_string() + "**\n" +
        "➤Stride: **" + &tlns_tetrio_calcs::truncate(player.stride(), 4).to_string() + "**\n" +
        "➤Infinite Downstack: **" + &tlns_tetrio_calcs::truncate(player.infinite_downstack(), 4).to_string() + "**\n"
        , true)
        .field("Want to know more?", "Check the calculation formulas code in https://github.com/timelessnesses/osker/blob/main/tlns-tetrio-calcs/src/lib.rs ! ^w^", true)
        .timestamp(poise::serenity_prelude::Timestamp::now())
}
