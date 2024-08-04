pub mod controller;
#[derive(Debug)]
pub struct PostgresDatabase {
    pub pool: sqlx::Pool<sqlx::Postgres>,
}

async fn sqlx_connect() -> sqlx::Pool<sqlx::Postgres> {
    //postgres://user:password@localhost:5432/dbname
    let db_url =
        std::env::var("DATABASE_URL").expect("Env var DATABASE_URL is required for this example.");
    use sqlx::postgres::PgPoolOptions;
    let pool = PgPoolOptions::new()
        .max_connections(50)
        .connect(db_url.as_str())
        .await
        .expect("Failed to connect to database");
    pool
}

impl PostgresDatabase {
    pub async fn new() -> Self {
        let pool = sqlx_connect().await;
        PostgresDatabase { pool }
    }
}
