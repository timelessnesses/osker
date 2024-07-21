#[derive(Debug, Clone, Copy)]
pub struct States {
    // database: std::sync::Arc<tokio_postgres::Client>,
    pub up_when: chrono::DateTime<chrono::Local>,
}