use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
    time::Duration,
};

use async_stream::stream;
use rspc::{
    integrations::httpz::{CookieJar, Request},
    BuiltRouter, ExportConfig, Rspc,
};

use tokio::time::sleep;

use crate::core::context::{self, Context};

fn cookies() -> context::middleware!() {
    |mw, mut ctx| async move {
        let request = context::query!(ctx, Mutex<Request>);
        let mut request = request.lock().unwrap();
        let cookies = request.cookies().ok_or_else(|| {
            rspc::Error::new(
                rspc::ErrorCode::InternalServerError,
                "Failed to find cookies in the request.".to_string(),
            )
        })?;

        context::add!(ctx, cookies);

        Ok(mw.next(ctx))
    }
}

fn auth() -> context::middleware!() {
    |mw, ctx| async move {
        let _cookies = context::query!(ctx, CookieJar);
        Ok(mw.next(ctx))
    }
}

pub const R: Rspc<Context> = Rspc::new();

pub fn get() -> Arc<BuiltRouter<Context>> {
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
