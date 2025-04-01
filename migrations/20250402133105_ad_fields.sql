ALTER table users
    ADD COLUMN dn TEXT UNIQUE NOT NULL,
    DROP COLUMN login,
    ALTER COLUMN email SET NOT NULL;

ALTER table hosts
    DROP CONSTRAINT hosts_user_id_fkey,
    ADD FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE SET NULL;
