use stand_control_bot::db::{run_migrations, Registry};
use stand_control_bot::logic::notifications::BotAdapter;
use stand_control_bot::{configuration::get_config, set_env, web::Application};
use tracing::info;

use ldap3::{drive, LdapConnAsync};
use secrecy::ExposeSecret;
use stand_control_bot::logic::release::hosts_release_timer;
use stand_control_bot::telemetry::init_tracing;

struct EmptyBotAdapter {}
impl BotAdapter for EmptyBotAdapter {
    async fn send_message(&self, _user_id: i64, _msg: String) -> anyhow::Result<()> {
        unreachable!()
    }
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let settings = get_config()?;
    init_tracing();
    set_env();

    run_migrations(&settings.database).await?;
    let registry = Registry::new(&settings.database).await?;

    let (ldap_conn, ldap) =
        LdapConnAsync::with_settings(settings.ldap.clone().into(), &settings.ldap.url).await?;
    let ldap_conn_task = drive!(ldap_conn);
    let (authorized_ldap_conn, mut authorized_ldap) =
        LdapConnAsync::with_settings(settings.ldap.clone().into(), &settings.ldap.url).await?;
    let authorized_ldap_conn_task = drive!(authorized_ldap_conn);
    authorized_ldap
        .simple_bind(&settings.ldap.login, settings.ldap.password.expose_secret())
        .await?
        .success()?;

    let server = Application::build(
        &settings,
        registry.clone(),
        ldap,
        authorized_ldap,
        "bot_username".into(),
    )
    .await?;

    tokio::select! {
        _ = server.serve_forever() => {
            info!("Server exited")
        }
        _ = ldap_conn_task => {
            info!("Ldap connection exited")
        }
        _ = authorized_ldap_conn_task => {
            info!("Authorized ldap connection exited")
        }
        _ = hosts_release_timer::<EmptyBotAdapter>(registry, &None) => {
            info!("Hosts release timer exited")
        }
    }
    Ok(())
}
