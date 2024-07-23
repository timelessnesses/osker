use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
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
                player.garbage_efficiency() * tlns_tetrio_calcs::weights::GARBAGE_EFFICIENCY_WEIGHT as f64
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
            "".to_string()
        );
        ctx.send(poise::CreateReply::default().attachment(
            poise::serenity_prelude::CreateAttachment::bytes(bytes, "stat.png"),
        ))
        .await?;
    }
    Ok(())
}