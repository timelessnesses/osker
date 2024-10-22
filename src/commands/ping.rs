use poise;

#[poise::command(prefix_command, slash_command)]
pub async fn ping(ctx: crate::types::Context<'_>) -> Result<(), crate::types::Error> {
    ctx.defer().await?;
    let shard = ctx.framework().shard_manager;
    let shard = shard.runners.lock().await;
    let ping = shard.get(&poise::serenity_prelude::ShardId(
        ctx.serenity_context().shard_id.0,
    ));
    if let Some(p) = ping {
        if let Some(d) = p.latency {
            let embed = poise::serenity_prelude::CreateEmbed::default()
                .title("Pong!")
                .description(format!(
                    "Shard Latency: {}ms\nShard ID: {}",
                    d.as_millis(),
                    ctx.serenity_context().shard_id.0
                ));
            ctx.send(poise::CreateReply::default().embed(embed)).await?;
        } else {
            return Err("Shard has no ping (Killed/Just started?)".into());
        }
    } else {
        return Err("Failed to fetch shard".into());
    }
    return Ok(());
}
