use stand_control_bot::db::{run_migrations, Registry};
use stand_control_bot::logic::message_senders::DisabledMessageSender;
use stand_control_bot::logic::notifications::Notifier;
use stand_control_bot::{configuration::get_config, set_env, web::Application};
use tracing::info;

use ldap3::{drive, LdapConnAsync};
use secrecy::ExposeSecret;
use stand_control_bot::logic::release::hosts_release_timer;
use stand_control_bot::telemetry::init_tracing;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let settings = get_config()?;
    init_tracing();
    set_env();

    run_migrations(&settings.database).await?;
    let registry = Registry::new(&settings.database).await?;
    let notifier = Notifier::new(registry.clone(), DisabledMessageSender {});

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
        _ = hosts_release_timer(registry, &notifier) => {
            info!("Hosts release timer exited")
        }
    }
    Ok(())
}
