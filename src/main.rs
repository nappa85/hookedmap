#![deny(warnings)]
#![deny(missing_docs)]

//! # hookedmap
//!
//! Map feeder via RocketMap webhooks

use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};

use futures_util::TryStreamExt;

use tokio::spawn;

use serde_json::value::Value;

use tracing::{debug, error, info};

mod config;
mod db;
mod engine;
mod lists;

async fn parse(bytes: Vec<u8>) -> Result<(), ()> {
    let body = String::from_utf8(bytes).map_err(|e| error!("encoding error: {}", e))?;
    // split the serialization in two passes, this way a single error doesn't break the entire block
    let configs: Vec<Value> = serde_json::from_str(&body).map_err(|e| error!("deserialize error: {}\n{}", e, body))?;

    engine::submit(configs.into_iter().flat_map(|v| {
        debug!("incoming webhook: {}", v);
        // this is a bit of a waste of memory, but there is no other way around
        serde_json::from_value(v.clone()).map_err(|e| error!("deserialize error: {}\n{}", e, v))
    }))
    .await;
    Ok(())
}

async fn service(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    if config::CONFIG.service.safeword.is_none()
        || Some(req.uri().path().trim_matches('/')) == config::CONFIG.service.safeword.as_deref()
    {
        let bytes = req.into_body().map_ok(|c| c.to_vec()).try_concat().await.map_err(|e| {
            error!("concat error: {}", e);
            e
        })?;

        //spawn an independent future to parse the stream
        spawn(async move {
            parse(bytes).await.ok();
        });
    }

    //always reply empty 200 OK
    Ok(Response::new(Body::empty()))
}

/// Launch service according to config
#[tokio::main]
async fn main() -> Result<(), ()> {
    tracing_subscriber::fmt::init();

    lists::init().await;

    //retrieve address and port, defaulting if not configured
    let addr = format!(
        "{}:{}",
        config::CONFIG.service.address.as_deref().unwrap_or("0.0.0.0"),
        config::CONFIG.service.port.unwrap_or(80)
    )
    .parse()
    .map_err(|e| error!("Error parsing webserver address: {}", e))?;

    //basic service function
    let service = make_service_fn(|_| async { Ok::<_, hyper::Error>(service_fn(service)) });

    info!("Starting webserver at {}", addr); //debug

    // bind and serve...
    Server::bind(&addr)
        .serve(service)
        .await
        .map_err(|e| {
            error!("server error: {}", e);
        })
        .ok();

    Ok(())
}
