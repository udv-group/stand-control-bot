mod auth;
mod hosts;

use askama_axum::IntoResponse;
use axum::{
    body::Body,
    extract::{FromRef, MatchedPath},
    middleware,
    response::Redirect,
    routing::{get, post},
    Router,
};

use axum_extra::extract::cookie::Key;
use axum_login::AuthManagerLayerBuilder;
use hyper::Request;
use std::net::SocketAddr;
use tokio::net::TcpListener;

use tower_http::trace::TraceLayer;
use tower_sessions::{cookie::time::Duration, Expiry, MemoryStore, SessionManagerLayer};
use tracing::info;
use uuid::Uuid;

use self::{
    auth::{
        login,
        middleware::{auth_middleware, Backend},
    },
    hosts::{get_hosts, lease_hosts, lease_random, release_all, release_hosts},
};
use crate::{
    configuration::Settings,
    db::Registry,
    logic::{hosts::HostsService, users::UsersService},
};

#[derive(FromRef, Clone)]
struct AppState {
    service: HostsService,
    users_service: UsersService,
    flash_config: axum_flash::Config,
    auth_link: AuthLink,
}

#[derive(Clone)]
pub struct AuthLink(pub String);

pub struct Application {
    listening_addr: SocketAddr,
    server: Server,
}

impl Application {
    pub async fn build(
        settings: &Settings,
        auth_link: String,
    ) -> Result<Application, anyhow::Error> {
        let tracing_layer = TraceLayer::new_for_http().make_span_with(|req: &Request<Body>| {
                let method = req.method();
                let uri = req.uri();
                let matched_path = req.extensions().get::<MatchedPath>().map(|p| p.as_str());

                tracing::debug_span!("http-request", %method, %uri, matched_path, request_id = %Uuid::new_v4())
            });

        let session_layer = SessionManagerLayer::new(MemoryStore::default())
            .with_secure(true)
            .with_expiry(Expiry::OnInactivity(Duration::days(7)));

        let registry = Registry::new(&settings.database).await?;

        let auth_layer = AuthManagerLayerBuilder::new(
            Backend::new(&settings.ldap, registry.clone()),
            session_layer,
        )
        .build();

        let authed_router = Router::new()
            .route("/logout", get(login::logout))
            .route("/hosts", get(get_hosts))
            .route("/hosts/lease", post(lease_hosts))
            .route("/hosts/lease/random", post(lease_random))
            .route("/hosts/release", post(release_hosts))
            .route("/hosts/release/all", post(release_all));

        let app = Router::new()
            .route("/login", post(login::login).get(login::login_page))
            .merge(authed_router.route_layer(middleware::from_fn(auth_middleware)))
            .fallback(|| async { Redirect::to("/hosts").into_response() })
            .layer(auth_layer)
            .layer(tracing_layer)
            .with_state(AppState {
                service: HostsService::new(registry.clone()),
                users_service: UsersService::new(registry),
                flash_config: axum_flash::Config::new(Key::derive_from(&settings.app.hmac_secret)),
                auth_link: AuthLink(auth_link),
            });

        let listener = TcpListener::bind(settings.app.socket_addr()).await?;
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
