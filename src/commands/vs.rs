use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};
use tlns_plotter;

/// Compares the stats of two users (or one) with more stats.
#[poise::command(prefix_command, slash_command)]
pub async fn vs(
    ctx: crate::types::Context<'_>,
    #[description = "Players string, supported"] players: Vec<String>,
) -> Result<(), crate::types::Error> {
    let is_stat = check_is_stat(&players);
    if players.len() == 1 || is_stat {
        let locked = ctx.data().player_lists.lock().await;
        let player = match is_stat {
            true => tlns_tetrio_calcs::ProfileStats::from_stat(
                players[0].parse().unwrap(),
                players[1].parse().unwrap(),
                players[2].parse().unwrap(),
            ),
            false => locked
                .par_iter()
                .find_first(|i| i.name.clone().unwrap().eq_ignore_ascii_case(&players[0]))
                .map(|i| i.clone())
                .unwrap_or(tlns_tetrio_calcs::ProfileStats::from_username(&players[0]).await?),
        };
        let bytes = tlns_plotter::plot_radar_one(
            [
                player.apm as f64 * tlns_tetrio_calcs::weights::APM_WEIGHT as f64,
                player.pps as f64 * tlns_tetrio_calcs::weights::PPS_WEIGHT as f64,
                player.vs as f64 * tlns_tetrio_calcs::weights::VS_WEIGHT,
                player.app() * tlns_tetrio_calcs::weights::APP_WEIGHT as f64,
                player.ds_seconds() * tlns_tetrio_calcs::weights::DS_SECONDS_WEIGHT as f64,
                player.ds_pieces() * tlns_tetrio_calcs::weights::DS_PIECES_WEIGHT as f64,
                player.app_ds_per_pieces() * tlns_tetrio_calcs::weights::DS_APP_WEIGHT as f64,
                player.vs_apm() * tlns_tetrio_calcs::weights::VS_APM_WEIGHT as f64,
                player.cheese_index() * tlns_tetrio_calcs::weights::CHEESE_INDEX_WEIGHT,
                player.garbage_efficiency()
                    * tlns_tetrio_calcs::weights::GARBAGE_EFFICIENCY_WEIGHT as f64,
            ],
            [
                "APM".to_string(),
                "PPS".to_string(),
                "VS".to_string(),
                "APP".to_string(),
                "DS/Seconds".to_string(),
                "DS/Pieces".to_string(),
                "APP+DS/Piece".to_string(),
                "VS/APM".to_string(),
                "Cheese Index".to_string(),
                "Garbage Efficiency".to_string(),
            ],
            "".to_string(),
        );
        ctx.send(poise::CreateReply::default().attachment(
            poise::serenity_prelude::CreateAttachment::bytes(bytes, "stat.png"),
        ))
        .await?;
    } else {
        let locked = ctx.data().player_lists.lock().await;
        let player = players
            .par_iter()
            .enumerate()
            .map(|(i, n)| {
                (
                    i,
                    locked
                        .par_iter()
                        .find_first(|i| i.name.clone().unwrap() == *n),
                )
            })
            .map(|(i, n)| {
                (
                    i,
                    match n {
                        Some(k) => Some(k.clone()),
                        None => None,
                    },
                )
            })
            .collect::<Vec<(usize, Option<tlns_tetrio_calcs::ProfileStats>)>>();
        let mut new_batch = Vec::new();
        for (i, p) in player {
            match p {
                Some(k) => new_batch.push(k),
                None => new_batch
                    .push(tlns_tetrio_calcs::ProfileStats::from_username(&players[i]).await?),
            }
        }
        let bytes = tlns_plotter::plot_radar_multiple(
            new_batch
                .iter()
                .map(|i| {
                    vec![
                        i.apm as f64 * tlns_tetrio_calcs::weights::APM_WEIGHT as f64,
                        i.pps as f64 * tlns_tetrio_calcs::weights::PPS_WEIGHT as f64,
                        i.vs as f64 * tlns_tetrio_calcs::weights::VS_WEIGHT,
                        i.app() * tlns_tetrio_calcs::weights::APP_WEIGHT as f64,
                        i.ds_seconds() * tlns_tetrio_calcs::weights::DS_SECONDS_WEIGHT as f64,
                        i.ds_pieces() * tlns_tetrio_calcs::weights::DS_PIECES_WEIGHT as f64,
                        i.app_ds_per_pieces() * tlns_tetrio_calcs::weights::DS_APP_WEIGHT as f64,
                        i.vs_apm() * tlns_tetrio_calcs::weights::VS_APM_WEIGHT as f64,
                        i.cheese_index() * tlns_tetrio_calcs::weights::CHEESE_INDEX_WEIGHT,
                        i.garbage_efficiency()
                            * tlns_tetrio_calcs::weights::GARBAGE_EFFICIENCY_WEIGHT as f64,
                    ]
                })
                .collect(),
            vec![
                "APM".to_string(),
                "PPS".to_string(),
                "VS".to_string(),
                "APP".to_string(),
                "DS/Seconds".to_string(),
                "DS/Pieces".to_string(),
                "APP+DS/Piece".to_string(),
                "VS/APM".to_string(),
                "Cheese Index".to_string(),
                "Garbage Efficiency".to_string(),
            ],
            new_batch.iter().map(|i| i.name.clone().unwrap()).collect(),
            "t".to_string(),
        );
        let colors = vec!["mint", "yellow", "blurple", "orange", "green", "purple"];
        ctx.send(
            poise::CreateReply::default()
                .attachment(poise::serenity_prelude::CreateAttachment::bytes(
                    bytes, "stat.png",
                ))
                .content(
                    new_batch
                        .iter()
                        .zip(colors.iter().cycle())
                        .map(|(i, c)| format!("{} is {} color", i.name.clone().unwrap(), c))
                        .collect::<Vec<String>>()
                        .join("\n"),
                ),
        )
        .await?;
    }
    Ok(())
}

fn check_is_stat(info: &Vec<String>) -> bool {
    info.par_iter().all(|i| i.parse::<i128>().is_ok()) && info.len() == 3
}
