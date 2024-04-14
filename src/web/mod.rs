mod hosts;

use axum::{
    body::Body,
    extract::{FromRef, MatchedPath},
    routing::get,
    Router,
};
use hyper::Request;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing::info;
use uuid::Uuid;

use self::hosts::get_hosts;
use crate::{configuration::Settings, db::Registry};

#[derive(FromRef, Clone)]
struct AppState {
    registry: Registry,
}

pub struct Application {
    listening_addr: SocketAddr,
    server: Server,
}

impl Application {
    pub async fn build(settings: &Settings) -> Result<Application, anyhow::Error> {
        let listener = TcpListener::bind(settings.app.socket_addr()).await?;
        let tracing_layer = TraceLayer::new_for_http().make_span_with(|req: &Request<Body>| {
                let method = req.method();
                let uri = req.uri();
                let matched_path = req.extensions().get::<MatchedPath>().map(|p| p.as_str());

                tracing::debug_span!("http-request", %method, %uri, matched_path, request_id = %Uuid::new_v4())
            });

        let app = Router::new()
            .route("/", get(get_hosts))
            .layer(tracing_layer)
            .with_state(AppState {
                registry: Registry::new(&settings.database).await?,
            });
        Ok(Self {
            listening_addr: listener.local_addr()?,
            server: Server::new(listener, app),
        })
    }
    pub async fn serve_forever(self) -> Result<(), std::io::Error> {
        info!("Web server is listening on {}", self.listening_addr);
        self.server.serve().await
    }
    pub fn listening_addr(&self) -> SocketAddr {
        self.listening_addr
    }
}

struct Server {
    listener: TcpListener,
    app: Router,
}
impl Server {
    pub fn new(listener: TcpListener, app: Router) -> Self {
        Self { listener, app }
    }

    pub async fn serve(self) -> Result<(), std::io::Error> {
        axum::serve(self.listener, self.app).await
    }
}
