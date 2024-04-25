use async_stream::stream;
use axum::Router;
use rspc::{
    internal::middleware::{ConstrainedMiddleware, SealedMiddleware},
    BuiltRouter, ExportConfig, Rspc,
};
use std::{
    error::Error,
    net::{Ipv6Addr, SocketAddr},
    path::PathBuf,
    sync::Arc,
    time::Duration,
};
use tokio::time::sleep;
use tower_http::cors::{Any, CorsLayer};

#[derive(Clone, Debug)]
struct Context {}

#[derive(Clone)]
struct ProtectedContext {}

const R: Rspc<Context> = Rspc::new();

macro_rules! middleware {
    ($context:ty, $new_ctx:ty) => {
        impl ConstrainedMiddleware<$context> + SealedMiddleware<$context, NewCtx = $new_ctx>
    }
}

fn auth() -> middleware!(Context, ProtectedContext) {
    |mw, _ctx| async move {
        println!("auth");
        mw.next(ProtectedContext {})
    }
}

fn router() -> Arc<BuiltRouter<Context>> {
    let version_query = R.with(auth()).query(|_ctx, _: ()| Ok("0.1.0"));

    let router = R
        .router()
        .procedure("version", version_query)
        .procedure("echo", R.query(|_, _: ()| Ok("0.1.0")))
        .procedure(
            "pings",
            R.subscription(|_, _: ()| {
                println!("Client subscribed to 'pings'");
                stream! {
                    yield Ok("start".to_string());
                    for i in 0..10 {
                        yield Ok(i.to_string());
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
        .allow_methods(Any)
        .allow_headers(Any)
        .allow_origin(Any);

    let router = router();
    let app = Router::new()
        .nest("/", router.endpoint(|| Context {}).axum())
        .layer(cors);

    let addr = SocketAddr::from((Ipv6Addr::UNSPECIFIED, 4000));

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
