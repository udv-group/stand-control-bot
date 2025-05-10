#!/usr/bin/env bash
set -x
set -eo pipefail

if ! [ -x "$(command -v psql)" ]; then
    echo >&2 "Error: psql is not installed."
    exit 1
fi
if ! [ -x "$(command -v sqlx)" ]; then
    echo >&2 "Error: sqlx is not installed."
    echo >&2 "Use:"
    echo >&2 " cargo install sqlx-cli --no-default-features --features postgres"
    echo >&2 "to install it."
    exit 1
fi

DB_USER="${POSTGRES_USER:=postgres}"
DB_PASSWORD="${POSTGRES_PASSWORD:=password}"
DB_NAME="${POSTGRES_DB:=tachikoma}"
DB_PORT="${POSTGRES_PORT:=5432}"

if [[ -z "${SKIP_START}" ]]
then
podman run \
    -e POSTGRES_USER=${DB_USER} \
    -e POSTGRES_PASSWORD=${DB_PASSWORD} \
    -e POSTGRES_DB=${DB_NAME} \
    -p "${DB_PORT}":5432 \
    --name postgres \
    -d docker.io/library/postgres:16.2-bullseye \
    postgres -N 1000
fi

# Keep pinging Postgres until it's ready to accept commands
export PGPASSWORD="${DB_PASSWORD}"
until psql -h "localhost" -U "${DB_USER}" -p "${DB_PORT}" -d "postgres" -c '\q'; do
    >&2 echo "Postgres is still unavailable - sleeping"
    sleep 1
done
    
>&2 echo "Postgres is up and running on port ${DB_PORT}!"
export DATABASE_URL=postgres://${DB_USER}:${DB_PASSWORD}@localhost:${DB_PORT}/${DB_NAME}
sqlx database create
sqlx migrate run
>&2 echo "Postgres has been migrated, ready to go!"

if [[ -z "${SKIP_SEED_HOSTS}" ]]
then
    PGPASSWORD="${POSTGRES_PASSWORD}" psql -h "localhost" -U "${DB_USER}" -p "${DB_PORT}" -d "${POSTGRES_DB}" -c "insert into hosts (hostname, ip_address, group_id) VALUES ('test', '172.0.0.1', 0), ('test2', '172.0.0.2', 0);"
fi
