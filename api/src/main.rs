use async_stream::stream;
use axum::{
    http::{
        header::{AUTHORIZATION, CONTENT_TYPE},
        HeaderValue, Method,
    },
    Router,
};
use cookie::Cookie;
use rspc::{
    integrations::httpz::{CookieJar, Request},
    internal::middleware::{ConstrainedMiddleware, SealedMiddleware},
    BuiltRouter, ErrorCode, ExportConfig, Rspc,
};
use std::{
    error::Error,
    net::{Ipv6Addr, SocketAddr},
    path::PathBuf,
    sync::Arc,
    time::Duration,
};
use tokio::time::sleep;
use tower_http::cors::CorsLayer;

#[derive(Clone, Debug)]
struct Context {
    cookies: Option<CookieJar>,
}

struct ContextWithCookies {
    cookies: CookieJar,
}

#[derive(Clone)]
struct ContextWithAuthentication {
    cookies: CookieJar,
}

const R: Rspc<Context> = Rspc::new();

macro_rules! middleware {
    ($context:ty, $new_ctx:ty) => {
        impl ConstrainedMiddleware<$context> + SealedMiddleware<$context, NewCtx = $new_ctx>
    }
}

fn cookies() -> middleware!(Context, ContextWithCookies) {
    |mw, ctx| async move {
        let cookies = ctx.cookies.ok_or_else(|| {
            rspc::Error::new(
                ErrorCode::InternalServerError,
                "Failed to find cookies in the request.".to_string(),
            )
        })?;

        Ok(mw.next(ContextWithCookies { cookies }))
    }
}

fn auth() -> middleware!(ContextWithCookies, ContextWithAuthentication) {
    |mw, ctx| async move {
        mw.next(ContextWithAuthentication {
            cookies: ctx.cookies,
        })
    }
}

fn router() -> Arc<BuiltRouter<Context>> {
    let version_query = R
        .with(cookies())
        .with(auth())
        .query(|_ctx, _: ()| Ok("0.1.0"));

    let router = R
        .router()
        .procedure("version", version_query)
        .procedure("echo", R.query(|_, _: ()| Ok("0.1.0")))
        .procedure(
            "pings",
            R.subscription(|_, _: ()| {
                println!("Client subscribed to 'pings'");
                stream! {
                    yield "start".to_string();
                    for i in 0..10 {
                        yield i.to_string();
                        sleep(Duration::from_secs(1)).await;
                    }
                }
            }),
        )
        .build()
        .unwrap()
        .arced();

    #[cfg(debug_assertions)]
    router
        .export_ts(ExportConfig::new(
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../web/app/generated/bindings.ts"),
        ))
        .unwrap();

    router
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_origin("http://localtest.me:5173".parse::<HeaderValue>().unwrap())
        .allow_headers([AUTHORIZATION, CONTENT_TYPE])
        .allow_credentials(true);

    let router = router();
    let app = Router::new()
        .nest(
            "/",
            router
                .endpoint(move |mut req: Request| Context {
                    cookies: req.cookies(),
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
