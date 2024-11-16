use std::net::{IpAddr, Ipv4Addr};

use fake::{Fake, Faker};
use ipnetwork::IpNetwork;
use sqlx::PgPool;

use rand::Rng;
use stand_control_bot::db::models::{GroupId, HostId, UserId};
use uuid::Uuid;

pub struct Generator {
    pub pool: PgPool,
}
pub struct MockHost {
    pub id: HostId,
    pub hostname: String,
    pub ip: Ipv4Addr,
}
pub struct MockGroup {
    pub id: GroupId,
    pub name: String,
}
pub struct MockUser {
    pub id: UserId,
    pub tg_handle: String,
}

impl Generator {
    pub async fn generate_group(&mut self) -> MockGroup {
        let name = Uuid::new_v4().to_string();
        let row = sqlx::query!("INSERT INTO groups (name) VALUES ($1) RETURNING id", name)
            .fetch_one(&self.pool)
            .await
            .unwrap();
        MockGroup {
            id: row.id.into(),
            name,
        }
    }
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
            hostname,
            ip,
        }
    }
    pub async fn generate_user(&mut self) -> MockUser {
        let mut rng = rand::thread_rng();

        let login = Uuid::new_v4().to_string();
        let tg_handle = rng.gen::<u64>().to_string();

        let row = sqlx::query!(
            "INSERT INTO users (login, tg_handle) VALUES ($1, $2) RETURNING id",
            login,
            tg_handle
        )
        .fetch_one(&self.pool)
        .await
        .unwrap();

        MockUser {
            id: row.id.into(),
            tg_handle,
        }
    }
}
