use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
    task::Poll,
};

use futures::Stream;
use rspc::{
    integrations::httpz::{CookieJar, Request},
    BuiltRouter, ExportConfig, Rspc,
};
use tokio::sync::mpsc;

use crate::core::{
    context::{self, Context},
    event::Emitter,
};

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
    let version_query = R.with(cookies()).with(auth()).query(|ctx, _: ()| {
        let emitter = context::query!(ctx, Arc<Emitter<i32>>);
        emitter.emit("test", 1);
        Ok("0.1.0")
    });

    let router = R
        .router()
        .procedure("version", version_query)
        .procedure("echo", R.query(|_, _: ()| Ok("0.1.0")))
        .procedure(
            "pings",
            R.subscription(|ctx, _: ()| {
                let (tx, rx) = mpsc::channel::<i32>(32);

                let emitter = context::query!(ctx, Arc<Emitter<i32>>);

                emitter.add_listener("test".to_string(), tx.clone());

                pub struct Subscription {
                    tx: mpsc::Sender<i32>,
                    rx: mpsc::Receiver<i32>,
                    emitter: Arc<Arc<Emitter<i32>>>,
                }

                impl Stream for Subscription {
                    type Item = i32;

                    fn poll_next(
                        mut self: std::pin::Pin<&mut Self>,
                        cx: &mut std::task::Context<'_>,
                    ) -> Poll<Option<Self::Item>> {
                        match self.rx.poll_recv(cx) {
                            Poll::Ready(Some(value)) => Poll::Ready(Some(value)),
                            Poll::Ready(None) => Poll::Ready(None),
                            Poll::Pending => Poll::Pending,
                        }
                    }
                }

                impl Drop for Subscription {
                    fn drop(&mut self) {
                        self.emitter.remove_listener("test".to_string(), &self.tx);
                    }
                }

                Ok(Subscription { rx, tx, emitter })
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
