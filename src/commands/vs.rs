use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};
use tlns_plotter;

/// Displays stats of a user in a table list.
#[poise::command(prefix_command, slash_command)]
pub async fn vs(
    ctx: crate::types::Context<'_>,
    #[description = "Players"] players: Vec<String>,
) -> Result<(), crate::types::Error> {
    if players.len() == 1 {
        let locked = ctx.data().player_lists.lock().await;
        let player = match locked
            .par_iter()
            .find_first(|i| i.name.clone().unwrap().to_lowercase() == players[0].to_lowercase())
        {
            Some(i) => i.clone(),
            None => tlns_tetrio_calcs::ProfileStats::from_username(&players[0]).await?,
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
            "".to_string(),
        );
        ctx.send(poise::CreateReply::default().attachment(
            poise::serenity_prelude::CreateAttachment::bytes(bytes, "stat.png"),
        ))
        .await?;
    }
    Ok(())
}
