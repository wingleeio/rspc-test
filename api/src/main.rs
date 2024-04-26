use axum::{
    http::{
        header::{AUTHORIZATION, CONTENT_TYPE},
        HeaderValue, Method,
    },
    Router,
};

use core::context::{self, Context};
use rspc::integrations::httpz::Request;
use std::{
    error::Error,
    net::{Ipv6Addr, SocketAddr},
};
use tokio::sync::mpsc;
use tower_http::cors::CorsLayer;

mod core;
mod router;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_origin("http://localtest.me:5173".parse::<HeaderValue>().unwrap())
        .allow_headers([AUTHORIZATION, CONTENT_TYPE])
        .allow_credentials(true);

    let router = router::get();

    let (tx, mut rx) = mpsc::channel::<String>(10);

    let app = Router::new()
        .nest(
            "/",
            router
                .endpoint(move |req: Request| {
                    let mut ctx = Context::new(req);

                    context::add!(ctx, tx.clone());

                    ctx
                })
                .axum(),
        )
        .layer(cors);

    let addr = SocketAddr::from((Ipv6Addr::UNSPECIFIED, 4000));

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
