use ajj::{HandlerCtx, ResponsePayload, Router};

use clap::Parser;
use log::info;

use crate::cli::Opts;

pub mod cli;

pub fn router() -> Router<()> {
    Router::<u64>::new()
        .route(
            "double",
            |params: u64| async move { Ok::<_, ()>(params * 2) },
        )
        .route("add", |params: u64, state: u64| async move {
            Ok::<_, ()>(params + state)
        })
        // Routes get a ctx, which can be used to send notifications.
        .route("notify", |ctx: HandlerCtx| async move {
            if ctx.notifications().is_none() {
                // This error will appear in the ResponsePayload's `data` field.
                return Err("notifications are disabled");
            }

            let req_id = 15u8;

            tokio::task::spawn_blocking(move || {
                // something expensive goes here
                let result = 100_000_000;
                let _ = ctx.notify(&serde_json::json!({
                  "req_id": req_id,
                  "result": result,
                }));
            });
            Ok(req_id)
        })
        .route("error_example", || async {
            // This will appear in the ResponsePayload's `message` field.
            ResponsePayload::<(), ()>::internal_error_message(
                "this is an error".into(),
            )
        })
        // The router is provided with state, and is now ready to serve requests.
        .with_state::<()>(3u64)
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    pretty_env_logger::init_timed();
    let opts = Opts::parse();

    let listener = tokio::net::TcpListener::bind(opts.address).await?;
    info!("Bound to {}", opts.address);
    axum::serve(listener, router().into_axum("/rpc")).await?;

    Ok(())
}
