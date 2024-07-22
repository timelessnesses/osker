
pub async fn connect_to_db() -> Result<sea_orm::DatabaseConnection, sea_orm::DbErr> {
    dotenv::dotenv().ok();
    let db_type = match std::env::var("OSKER_DB_TYPE")
        .expect("OSKER_DB_TYPE is not found")
        .as_str()
    {
        "postgres" | "postgresql" => "postgres",
        "mysql" => "mysql",
        "sqlite" | "local" => "sqlite",
        _ => panic!("Not in supported databases"),
    };
    if db_type == "sqlite" {
        let file = std::env::var("OSKER_DB_PATH").expect("OSKER_DB_PATH is not found");
        sea_orm::Database::connect(format!("sqlite://{file}?mode=rwc")).await
    } else {
        let host = std::env::var("OSKER_DB_HOST").expect("OSKER_DB_HOST is not found");
        let username = std::env::var("OSKER_DB_USERNAME").expect("OSKER_DB_USERNAME is not found");
        let password = std::env::var("OSKER_DB_PASSWORD").expect("OSKER_DB_PASSWORD is not found");
        let port = std::env::var("OSKER_DB_PORT")
            .expect("OSKER_DB_PORT")
            .parse::<u16>()
            .expect("Failed to parse integer");
        let database = std::env::var("OSKER_DB").expect("OSKER_DB is not found");
        sea_orm::Database::connect(format!(
            "{db_type}://{username}:{password}@{host}:{port}/{database}"
        ))
        .await
    }
}
