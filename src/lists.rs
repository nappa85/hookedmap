use std::{collections::HashMap, sync::Arc};

use arc_swap::ArcSwap;

use futures_util::TryStreamExt;

use geo::{Point, Polygon};

use mysql_async::{
    prelude::{FromRow, Queryable},
    Row,
};

use tokio::time::{interval_at, Duration, Instant};

use once_cell::sync::Lazy;

use tracing::error;

use crate::db::get_conn;

pub static CITIES: Lazy<ArcSwap<HashMap<u16, City>>> = Lazy::new(Default::default);

#[allow(dead_code)]
pub struct City {
    pub id: u16,
    pub name: String,
    pub coordinates: Polygon<f64>,
    pub scadenza: i64,
    pub scan_iv: u8,
    pub admins_users: Vec<String>,
}

impl FromRow for City {
    fn from_row_opt(mut row: Row) -> Result<Self, mysql_async::FromRowError> {
        let id = row.take("id").expect("MySQL city.id error");
        let name = row.take("name").expect("MySQL city.name error");
        let coords = row.take::<String, _>("coordinates").expect("MySQL city.coordinates encoding error");
        let coords = coords.replace(char::is_whitespace, "");

        let poly: Vec<Point<f64>> = if coords.len() < 2 {
            error!("City \"{}\" ({}) has empty coordinates", name, id);
            Vec::new()
        } else {
            coords[1..(coords.len() - 1)]
                .split("),(")
                .filter_map(|s| {
                    let x_y: Vec<f64> = s
                        .split(',')
                        .map(|s| match s.parse::<f64>() {
                            Ok(f) => f,
                            Err(_) => panic!("Error parsing \"{}\" as a float", s),
                        })
                        .collect();
                    if x_y.len() == 2 {
                        Some(Point::new(x_y[0], x_y[1]))
                    } else {
                        error!("City \"{}\" ({}) has invalid coordinates", name, id);
                        None
                    }
                })
                .collect()
        };

        Ok(City {
            id,
            name,
            coordinates: Polygon::new(poly.into(), vec![]),
            scadenza: row.take("scadenza").expect("MySQL city.scadenza error"),
            scan_iv: row.take("monitor").expect("MySQL city.monitor error"),
            admins_users: row
                .take::<String, _>("admins_users")
                .expect("MySQL city.admins_users error")
                .split_whitespace()
                .map(|s| s.to_owned())
                .collect(),
        })
    }
}

pub async fn load_cities() -> Result<(), ()> {
    let mut conn = get_conn().await?;
    let res = conn
        .query_iter("SELECT id, name, coordinates, scadenza, monitor, admins_users FROM city")
        .await
        .map_err(|e| error!("MySQL query error: get cities\n{}", e))?;

    let data = res
        .stream_and_drop::<City>()
        .await
        .map_err(|e| error!("MySQL load_cities error: {}", e))?
        .ok_or_else(|| error!("MySQL load_cities empty"))?
        .map_ok(|c| (c.id, c))
        .try_collect()
        .await
        .map_err(|e| error!("MySQL load_cities collect error: {}", e))?;
    CITIES.swap(Arc::new(data));

    Ok(())
}

pub async fn init() {
    // force first load
    load_cities().await.unwrap();
    tokio::spawn(async {
        let period = Duration::from_secs(1800);
        let mut interval = interval_at(Instant::now() + period, period);
        loop {
            interval.tick().await;
            load_cities().await.ok();
        }
    });
}
