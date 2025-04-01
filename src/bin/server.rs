use stand_control_bot::db::run_migrations;
use stand_control_bot::{configuration::get_config, set_env, web::Application};
use tracing::info;

use ldap3::{drive, LdapConnAsync};
use stand_control_bot::telemetry::init_tracing;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let settings = get_config()?;
    init_tracing();
    set_env();

    run_migrations(&settings.database).await?;

    let (conn, ldap) =
        LdapConnAsync::with_settings(settings.ldap.clone().into(), &settings.ldap.url).await?;

    let conn_task = drive!(conn);
    let server = Application::build(&settings, ldap, "bot_username".into()).await?;

    tokio::select! {
        _ = server.serve_forever() => {
            info!("Server exited")
        }
        _ = conn_task => {
            info!("Ldap connection exited")
        }
    }
    Ok(())
}
