use axum::
    Router
;
use std::net::SocketAddr;
use tower_http::{
    services::ServeDir,
    trace::TraceLayer,
};

use crate::context::get_context;

pub fn start() {

    let ctx = get_context();
    let deployment_path = ctx.config.deployment_folder.clone();
    drop(ctx);

    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {

            let app = Router::new().nest_service(
                "/",
                ServeDir::new(deployment_path),
            );

            let addr = SocketAddr::from(([127, 0, 0, 1], 3001));
            let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
            println!("listening on {}", listener.local_addr().unwrap());
            axum::serve(listener, app.layer(TraceLayer::new_for_http()))
                .await
                .unwrap();
        })
}
