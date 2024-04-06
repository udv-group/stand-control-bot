CREATE TABLE users (
    id serial PRIMARY KEY,
    login TEXT UNIQUE NOT NULL,
    tg_handle TEXT NULL,
    email TEXT NULL
);

CREATE TABLE hosts (
    id serial PRIMARY KEY,
    hostname TEXT NOT NULL,
    ip_address inet NOT NULL,
    leased_until timestamptz NULL,
    user_id integer REFERENCES users (id)
);