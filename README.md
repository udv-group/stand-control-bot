# STAND-CONTROL-BOT

Simple and opinionated web server to facilitate the reuse of development servers and infrastructure. 

# Deployment

[Releases](https://github.com/udv-group/stand-control-bot/releases) page has two prebuild binaries for Linux: `stand-control-bot` which includes telegram integration and `sever` which just runs a web server.

App is expected to be deployed in the private network. It is recommended to use a reverse proxy (like Nginx) to enable TLS and Docker to run a web server. For integration with telegram also network access
to telegram servers is required.

Web server is using LDAP to manage auth and is designed to be used with AD servers. 

## Configuration

### Env

`APP_ENVIRONMENT` - what environment app is running in. Possible values are `local` (default) and `production`. Affects configuration lookup.

`CONFIG_DIR` - path to directory with configuration files, default is `/etc/stand-control-bot` for `production` environment.

`INCLUDE_SPAN_EVENTS` - flag to include span events in log output, making it much more verbose. Possible values are `true` and `false` (default).

`TELOXIDE_TOKEN` - telegram bot token to enable telegram notifications. Can be obtained from [@Botfather](https://t.me/botfather) bot on telegram.

### Configuration files

Consult [base.toml](configuration/base.toml) for a list of all possible configuration options.

Both `base.toml` and `production.toml` (or `local.toml` for `APP_ENVIRONMENT=local`) need to be present in `CONFIG_DIR`.

Configuration presidence is as follows: `env` > `production.toml`/`local.toml` > `base.toml`. 


# Development

Prerequisites:
- Up to date Rust compiler ([rustup](https://www.rust-lang.org/tools/install) is strongly recommended)
- Postgres (>=13)
- [sqlx-cli](https://github.com/launchbadge/sqlx/blob/main/sqlx-cli/README.md)
- LDAP server

To initialize the database and run migrations use [init_db.sh](scripts/init_db.sh) script. 
If you don't have podman or running a native server you can skip the initialization stage with `SKIP_START=true` environment variable to just run migrations.

To set up a local LDAP server consult [ldap3 repo](https://github.com/inejge/ldap3/tree/00a513ece4ffa9a9782860c285f4c4c12bc07552/data). If run as is, authentication with empty credentials is possible.

To allow sqlx to connect to your development database create `.env` file in the root of this repository with a line 
```
DATABASE_URL=postgres://postgres:password@localhost:5432/stand_control_bot
```
Replace `localhost:5432` with the address and port of your postgres server

### Notes about LDAP/AD server

It is preferable to use a real AD server to run auth tests since auth is designed to
work with AD specifically.

It is possible to use LDAP-server instead, though you will have to use fully qualified names as logins.
