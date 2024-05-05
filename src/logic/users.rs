use thiserror::Error;

use crate::db::{models::User, Registry};

#[derive(Error, Debug)]
pub enum UserError {
    #[error("Database error")]
    DatabaseError(#[from] sqlx::Error),
}

#[derive(Clone)]
pub struct UsersService {
    registry: Registry,
}

impl UsersService {
    pub fn new(registry: Registry) -> Self {
        UsersService { registry }
    }

    pub async fn link_user(&self, link: &str, tg_user_id: &str) -> Result<Option<User>, UserError> {
        let mut tx = self.registry.begin().await?;
        match tx.get_user_by_link(link).await? {
            Some(user) => {
                tx.set_user_tg_handle(&user.id, tg_user_id).await?;
                let user = tx.get_user_by_id(&user.id).await?.unwrap();
                tx.commit().await?;
                Ok(Some(user))
            }
            None => Ok(None),
        }
    }
}
