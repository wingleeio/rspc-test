use axum::{
    http::{
        header::{AUTHORIZATION, CONTENT_TYPE},
        HeaderValue, Method,
    },
    Router,
};

use core::context::Context;
use rspc::integrations::httpz::Request;
use std::{
    error::Error,
    net::{Ipv6Addr, SocketAddr},
};
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
    let app = Router::new()
        .nest(
            "/",
            router
                .endpoint(move |req: Request| Context::new(req))
                .axum(),
        )
        .layer(cors);

    let addr = SocketAddr::from((Ipv6Addr::UNSPECIFIED, 4000));

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
