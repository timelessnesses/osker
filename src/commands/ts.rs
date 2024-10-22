use std::str::FromStr;

use poise;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use regex;

const FUNNY_IMAGE: &str = "https://statics.timelessnesses.me/poiuu_drawings/sd.png";
const DETECT_AVG_PATTERN: &str = r"(?m)\$avg(ALL|[a-zA-Z])(.*)"; // my genius brain made this yay
pub const ZERO_WIDTH_SPACE: &str = "\u{200b}";

/// Displays stats of a user in a table list.
#[poise::command(prefix_command, slash_command)]
pub async fn ts(
    ctx: crate::types::Context<'_>,
    #[description = "Argument that supports custom 'APM PPS VS' value or username or $avg`Rank or ALL`"]
    args: Vec<String>,
) -> Result<(), crate::types::Error> {
    ctx.defer().await?;
    let player: tlns_tetrio_calcs::ProfileStats;
    let mut should_add = false;
    let mut is_avg_rank = false;
    let mut fetched_from_api = false;

    {
        let players_list = ctx.data().player_lists.read().await;
        let average_players = ctx.data().avg_players.read().await;

        if args.len() == 3 {
            player = tlns_tetrio_calcs::ProfileStats::from_stat(
                args[0].parse()?,
                args[1].parse()?,
                args[2].parse()?,
            );
        } else {
            let r = regex::Regex::new(DETECT_AVG_PATTERN)?;
            if let Some(_) = r.captures(&args[0]) {
                player = if let Some(avg) = average_players
                    .iter()
                    .find(|i| *i.name.as_ref().unwrap() == args[0])
                {
                    is_avg_rank = true;
                    avg.clone()
                } else {
                    return Err(crate::errors::Errors::RankNotFoundError.into());
                };
            } else if let Some(p) = players_list
                .par_iter()
                .find_first(|i| i.name.clone().unwrap_or_default() == args[0])
            {
                player = p.clone();
            } else {
                player = tlns_tetrio_calcs::ProfileStats::from_username(&args[0]).await?;
                should_add = true;
                fetched_from_api = true;
            }
        }
    }
    if should_add {
        ctx.data().player_lists.write().await.push(player.clone());
    }

    let embed = build_player_embed(
        &player,
        match is_avg_rank {
            true => Some(format!("AVERAGE STATS ON RANK {}", player.rank.unwrap())),
            false => None,
        },
        fetched_from_api,
    );
    ctx.send(poise::CreateReply::default().embed(embed).reply(true))
        .await?;
    // ctx.say(format!("This request is from ch.tetr.io API: {fetched_from_api}")).await?;
    Ok(())
}

fn build_player_embed(
    player: &tlns_tetrio_calcs::ProfileStats,
    custom_title: Option<String>,
    is_fetched_from_api: bool,
) -> poise::serenity_prelude::CreateEmbed {
    let sign = if player.accuracy_tr() > 0.0 { "+" } else { "" };
    poise::serenity_prelude::CreateEmbed::new()
        .colour(poise::serenity_prelude::Color::from_rgb(0, 153, 255))
        .title(match custom_title {
            Some(m) => m,
            None => match player.is_real {
                true => player.name.as_ref().unwrap().to_string(),
                false => {
                    format!(
                        "ADVANCED STATS OF [{}, {}, {}]",
                        player.apm, player.pps, player.vs
                    )
                }
            }
        })
        .url(match player.is_real {
            true => "https://ch.tetr.io/u/".to_string() + player.name.as_ref().unwrap(),
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
                "➤Estimated TR Accuracy: **" + sign + &tlns_tetrio_calcs::truncate(player.accuracy_tr(), 2).to_string() + "**\n" +
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
        .footer(poise::serenity_prelude::CreateEmbedFooter::new(match is_fetched_from_api {
            true => "This command used ch.tetr.io API",
            false => "This command used it's own interal cache (that is refreshed every 5 minutes)"
        }))
}
// i love men
