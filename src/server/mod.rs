use std::{net::SocketAddr, sync::Arc, time::Instant};

use axum::{extract::Path, routing::get, Extension, Router};
use tokio::sync::{mpsc, oneshot};

struct AppState {
    tx: mpsc::Sender<(String, oneshot::Sender<String>)>,
}

pub struct Server {}

async fn root() -> String {
    "Nothing to see here".to_string()
}

async fn execute_function(
    Path(path): Path<String>,
    Extension(state): Extension<Arc<AppState>>,
) -> String {
    let now = Instant::now();
    let tx = state.clone().tx.clone();
    drop(state);
    let (tx_oneshot, rx) = oneshot::channel();
    tx.send((path.clone(), tx_oneshot)).await.unwrap();

    match rx.await {
        Ok(_) => {
            println!(
                "Handler: Executing {} took {}ms",
                path,
                now.elapsed().as_millis()
            );
            "ok".to_string()
        }
        Err(_) => "sad".to_string(),
    }
}

impl Server {
    pub async fn start(tx: mpsc::Sender<(String, oneshot::Sender<String>)>) {
        let app_state = Arc::new(AppState { tx });

        let app = Router::new()
            .route("/", get(root))
            .route("/:app", get(execute_function))
            .layer(Extension(app_state));

        let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

        let _server = axum::Server::bind(&addr)
            .serve(app.into_make_service())
            .await;
    }
}
