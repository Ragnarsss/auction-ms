use sea_orm::{Database, DatabaseConnection};
use std::env;

pub async fn connect() -> DatabaseConnection {
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL no seteado");
    log::info!("Conectando a la base de datos: {}", db_url);
    Database::connect(&db_url).await.expect("No se pudo conectar a la BD")
}

