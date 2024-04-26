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
    sync::Mutex,
};
use tower_http::cors::CorsLayer;

mod core;
mod router;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let addr = SocketAddr::from((Ipv6Addr::UNSPECIFIED, 4000));

    let router = router::get();

    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_origin("http://localtest.me:5173".parse::<HeaderValue>().unwrap())
        .allow_headers([AUTHORIZATION, CONTENT_TYPE])
        .allow_credentials(true);

    let emitter = core::event::Emitter::<i32>::new();

    let app = Router::new()
        .nest(
            "/",
            router
                .endpoint(move |req: Request| {
                    let mut ctx = Context::new();

                    context::add!(ctx, Mutex::new(req));
                    context::add!(ctx, emitter);

                    ctx
                })
                .axum(),
        )
        .layer(cors);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
