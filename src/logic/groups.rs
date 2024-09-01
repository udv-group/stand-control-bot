use thiserror::Error;

use crate::db::{models::Group, Registry};

#[derive(Error, Debug)]
pub enum GroupError {
    #[error("Database error")]
    DatabaseError(#[from] sqlx::Error),
}

#[derive(Clone)]
pub struct GroupsService {
    registry: Registry,
}

impl GroupsService {
    pub fn new(registry: Registry) -> Self {
        GroupsService { registry }
    }

    pub async fn get_all_groups(&self) -> Result<Vec<Group>, GroupError> {
        let mut tx = self.registry.begin().await?;
        let groups = tx.get_groups().await?;
        tx.commit().await?;
        Ok(groups)
    }
}
