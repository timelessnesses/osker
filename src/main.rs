use poise;
use serenity;
use dotenv;
use tokio;
use chrono;

mod db;
mod types;
mod state;

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