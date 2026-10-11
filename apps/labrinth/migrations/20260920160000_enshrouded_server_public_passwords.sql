CREATE TABLE enshrouded_server_public_passwords (
	project_id BIGINT NOT NULL REFERENCES mods(id) ON DELETE CASCADE,
	group_name VARCHAR(64) NOT NULL,
	password VARCHAR(128) NOT NULL,
	PRIMARY KEY (project_id, group_name),
	CONSTRAINT enshrouded_server_public_passwords_group_name_not_blank
		CHECK (BTRIM(group_name) <> ''),
	CONSTRAINT enshrouded_server_public_passwords_password_not_blank
		CHECK (BTRIM(password) <> '')
);
