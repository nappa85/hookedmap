use mysql_async::{Conn, Pool};

use once_cell::sync::Lazy;

use tracing::error;

use crate::config::CONFIG;

static MYSQL: Lazy<Pool> = Lazy::new(|| Pool::new(CONFIG.database.url.as_str()));

pub async fn get_conn() -> Result<Conn, ()> {
    MYSQL.get_conn().await.map_err(|e| error!("MySQL connection error: {}", e))
}
