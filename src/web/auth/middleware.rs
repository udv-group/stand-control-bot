use crate::db::models::User as DbUser;
use crate::db::Registry;
use crate::ldap::UsersInfo;
use anyhow::Context;
use axum::response::IntoResponse;
use axum::response::{Redirect, Response};
use axum::{async_trait, extract::Request, middleware::Next};
use axum_login::{AuthUser, AuthnBackend, UserId};
use ldap3::Ldap;
use ldap3::LdapError;
use secrecy::{ExposeSecret, Secret};
use tracing::info;

#[derive(Debug, Clone)]
pub struct User {
    id: i32,
    pub username: String,
    pub groups: Vec<String>,
    pub tg_handle: Option<String>,
    pub link: String,
    session_token: Vec<u8>,
}

impl From<(DbUser, Vec<String>)> for User {
    fn from(value: (DbUser, Vec<String>)) -> Self {
        let (user, groups) = value;
        Self {
            id: *user.id,
            username: user.email,
            tg_handle: user.tg_handle,
            link: user.link.clone(),
            session_token: user.link.into_bytes(),
            groups,
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
    users_info: UsersInfo,
    ldap: Ldap,
    registry: Registry,
}

impl Backend {
    pub fn new(ldap: Ldap, registry: Registry, users_info: UsersInfo) -> Self {
        Backend {
            users_info,
            ldap,
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
    #[error("Unexpected error: {0}")]
    AnyhowErr(#[from] anyhow::Error),
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
        let u_info = self.users_info.find_user_info(username.clone()).await?;
        if u_info.is_none() {
            info!("Authentication failed for '{}': unknown user", username);
            return Ok(None);
        }

        let u_info = u_info.unwrap();
        let mut ldap = self.ldap.clone();
        let resp = ldap
            .simple_bind(&u_info.dn, password.expose_secret())
            .await
            .and_then(|r| r.success());

        if let Err(err) = resp {
            info!("Authentication failed for '{}': '{}'", username, err);
            return Ok(None);
        }
        let user = self
            .registry
            .begin()
            .await?
            .get_user_by_dn(&u_info.dn)
            .await?;
        let user = match user {
            Some(u) => u,
            None => {
                let mut tx = self.registry.begin().await?;
                tx.add_user(&u_info.dn, None, &u_info.email).await?;
                let user = tx.get_user_by_dn(&u_info.dn).await?;
                tx.commit().await?;
                user.unwrap()
            }
        };
        Ok(Some((user, u_info.groups).into()))
    }

    async fn get_user(&self, user_id: &UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
        let user_id = *user_id;
        let user = self
            .registry
            .begin()
            .await?
            .get_user_by_id(&user_id.into())
            .await?;

        if user.is_none() {
            return Ok(None);
        }

        let user = user.unwrap();
        let u_info = self
            .users_info
            .get_user_info(&user.dn)
            .await?
            .with_context(|| format!("Missed user info '{}' ({})", user.dn, user.email))?;

        Ok(Some((user, u_info.groups).into()))
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
