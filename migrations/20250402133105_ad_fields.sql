ALTER table hosts
    DROP CONSTRAINT hosts_user_id_fkey,
    ADD FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE SET NULL;

DELETE FROM users;
ALTER table users
    ADD COLUMN dn TEXT UNIQUE NOT NULL,
    DROP COLUMN login,
    ALTER COLUMN email SET NOT NULL;


CREATE table lease_limits_by_ad_group (
    "group" TEXT PRIMARY KEY,
    "limit" smallint NOT NULL
);
