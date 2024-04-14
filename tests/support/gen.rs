use std::net::{IpAddr, Ipv4Addr};

use fake::{Fake, Faker};
use ipnetwork::IpNetwork;
use sqlx::PgPool;

use stand_control_bot::db::models::{HostId, UserId};
use uuid::Uuid;

pub struct Generator {
    pub pool: PgPool,
}
pub struct MockHost {
    pub id: HostId,
    pub ip: Ipv4Addr,
}
impl Generator {
    pub async fn generate_host(&mut self) -> MockHost {
        let hostname = Uuid::new_v4().to_string();
        let ip: Ipv4Addr = Faker.fake();
        let net = IpNetwork::new(IpAddr::V4(ip), 32).unwrap();
        let row = sqlx::query!(
            "INSERT INTO hosts (hostname, ip_address) VALUES ($1, $2) RETURNING id",
            hostname,
            net
        )
        .fetch_one(&self.pool)
        .await
        .unwrap();
        MockHost {
            id: row.id.into(),
            ip,
        }
    }
    pub async fn generate_user(&mut self) -> UserId {
        let login = Uuid::new_v4().to_string();
        let row = sqlx::query!("INSERT INTO users (login) VALUES ($1) RETURNING id", login)
            .fetch_one(&self.pool)
            .await
            .unwrap();
        row.id.into()
    }
}
