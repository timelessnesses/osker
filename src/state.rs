#[derive(Debug, Clone)]
pub struct States {
    // database: std::sync::Arc<tokio_postgres::Client>,
    pub up_when: chrono::DateTime<chrono::Local>,
    pub player_lists: std::sync::Arc<tokio::sync::RwLock<Vec<tlns_tetrio_calcs::ProfileStats>>>,
    pub avg_players: std::sync::Arc<tokio::sync::RwLock<Vec<tlns_tetrio_calcs::ProfileStats>>>,
}
