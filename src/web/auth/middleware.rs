use axum::response::IntoResponse;
use axum::response::{Redirect, Response};
use axum::{async_trait, extract::Request, middleware::Next};
use axum_login::{AuthUser, AuthnBackend, UserId};
use ldap3::{LdapConnAsync, LdapConnSettings, LdapError};
use secrecy::{ExposeSecret, Secret};
use tracing::info;

use crate::configuration::LdapSettings;
use crate::db::models::User as DbUser;
use crate::db::Registry;

#[derive(Debug, Clone)]
pub struct User {
    id: i32,
    pub username: String,
    pub tg_handle: Option<String>,
    pub link: String,
    session_token: Vec<u8>,
}

impl From<DbUser> for User {
    fn from(user: DbUser) -> Self {
        Self {
            id: *user.id,
            username: user.login,
            tg_handle: user.tg_handle,
            link: user.link.clone(),
            session_token: user.link.into_bytes(),
        }
    }
}

impl AuthUser for User {
    type Id = i32;

    fn id(&self) -> Self::Id {
        self.id
    }

    fn session_auth_hash(&self) -> &[u8] {
        &self.session_token
    }
}

#[derive(Clone)]
pub struct Backend {
    ldap_uri: String,
    ldap_settings: LdapConnSettings,
    registry: Registry,
}

impl Backend {
    pub fn new(settings: &LdapSettings, registry: Registry) -> Self {
        Backend {
            ldap_uri: settings.url.clone(),
            ldap_settings: LdapConnSettings::new()
                .set_no_tls_verify(settings.no_tls_verify)
                .set_starttls(settings.use_tls),
            registry,
        }
    }
}

#[derive(Clone)]
pub struct Credentials {
    pub username: String,
    pub password: Secret<String>,
}

#[derive(thiserror::Error, Debug)]
pub enum AuthError {
    #[error("Failed to connec to to ldap")]
    LdapConnError(#[from] LdapError),
    #[error("Database error")]
    DbError(#[from] sqlx::Error),
}

#[async_trait]
impl AuthnBackend for Backend {
    type User = User;
    type Credentials = Credentials;
    type Error = AuthError;

    async fn authenticate(
        &self,
        Credentials { username, password }: Credentials,
    ) -> Result<Option<Self::User>, Self::Error> {
        let (conn, mut ldap) =
            LdapConnAsync::with_settings(self.ldap_settings.clone(), &self.ldap_uri).await?;
        ldap3::drive!(conn);
        // NB: slapd does not allow AD-style login with jon.doe@example.com
        let resp = ldap
            .simple_bind(&username, password.expose_secret())
            .await
            .and_then(|r| r.success());
        ldap.unbind().await?;

        if let Err(err) = resp {
            info!("Authentication failed for '{}': '{}'", username, err);
            return Ok(None);
        }
        let user = self.registry.begin().await?.get_user(&username).await?;
        let user = match user {
            Some(u) => u,
            None => {
                let mut tx = self.registry.begin().await?;
                tx.add_user(&username, None, None).await?;
                let user = tx.get_user(&username).await?;
                tx.commit().await?;
                user.unwrap()
            }
        };
        Ok(Some(user.into()))
    }

    async fn get_user(&self, user_id: &UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
        let user_id = *user_id;
        Ok(self
            .registry
            .begin()
            .await?
            .get_user_by_id(&user_id.into())
            .await?
            .map(|u| u.into()))
    }
}

pub async fn auth_middleware(
    auth_session: AuthSession,
    mut request: Request,
    next: Next,
) -> Response {
    if let Some(user) = auth_session.user {
        let span = tracing::Span::current();
        span.record("username", tracing::field::display(&user.username));
        span.record("user_id", tracing::field::display(&user.id));
        request.extensions_mut().insert(user);
        next.run(request).await
    } else {
        Redirect::to("/login").into_response()
    }
}

pub type AuthSession = axum_login::AuthSession<Backend>;
