-- Dummy test data for use in tests.
-- IDs are listed as integers, followed by their equivalent base 62 representation.

-- Inserts 5 dummy users for testing, with slight differences
-- 'Friend' and 'enemy' function like 'user', but we can use them to simulate 'other' users that may or may not be able to access certain things
INSERT INTO users (id, username, email, role)
VALUES ({{user_id::ADMIN}},     'Admin',     'admin@shroudedit.com',     'admin'),
       ({{user_id::MODERATOR}}, 'Moderator', 'moderator@shroudedit.com', 'moderator'),
       ({{user_id::USER}},      'User',      'user@shroudedit.com',      'developer'),
       ({{user_id::FRIEND}},    'Friend',    'friend@shroudedit.com',    'developer'),
       ({{user_id::ENEMY}},     'Enemy',     'enemy@shroudedit.com',     'developer');

-- Full PATs for each user, with different scopes
-- These are not legal PATs, as they contain all scopes- they mimic permissions of a logged in user
-- IDs: 50-54, o p q r s
INSERT INTO pats (id, user_id, name, access_token, scopes, expires)
VALUES (50, {{user_id::ADMIN}},     'admin-pat',     '{{pat::ADMIN}}',     {{all_scopes}}, '2030-08-18 15:48:58.435729+00'),
       (51, {{user_id::MODERATOR}}, 'moderator-pat', '{{pat::MODERATOR}}', {{all_scopes}}, '2030-08-18 15:48:58.435729+00'),
       (52, {{user_id::USER}},      'user-pat',      '{{pat::USER}}',      {{all_scopes}}, '2030-08-18 15:48:58.435729+00'),
       (53, {{user_id::FRIEND}},    'friend-pat',    '{{pat::FRIEND}}',    {{all_scopes}}, '2030-08-18 15:48:58.435729+00'),
       (54, {{user_id::ENEMY}},     'enemy-pat',     '{{pat::ENEMY}}',     {{all_scopes}}, '2030-08-18 15:48:58.435729+00');

INSERT INTO loaders_project_types_games (loader_id, project_type_id, game_id)
SELECT loaders_project_types.joining_loader_id,
	loaders_project_types.joining_project_type_id,
	games.id
FROM loaders_project_types
JOIN loaders ON loaders.id = loaders_project_types.joining_loader_id
CROSS JOIN games
WHERE loaders.loader IN ('shroudforge', 'eml', 'shroudtopia')
	AND games.name = 'enshrouded'
ON CONFLICT DO NOTHING;

-- Adds valid Enshrouded loaders to modpack_loaders.
INSERT INTO loader_field_enum_values (enum_id, value)
SELECT loader_field_enums.id, supported_loaders.value
FROM loader_field_enums
CROSS JOIN (VALUES ('shroudforge'), ('eml'), ('shroudtopia')) AS supported_loaders(value)
WHERE loader_field_enums.enum_name = 'modpack_loaders'
ON CONFLICT DO NOTHING;

-- Dummy-data-only optional field for loader field validation tests
INSERT INTO loader_fields
	(field, field_type, optional)
VALUES
	('test_shroudforge_optional', 'integer', TRUE);
INSERT INTO loader_fields_loaders (loader_id, loader_field_id)
SELECT l.id, lf.id
FROM
	loaders AS l
	CROSS JOIN loader_fields AS lf
WHERE lf.field = 'test_shroudforge_optional' AND l.loader = 'shroudforge'
ON CONFLICT DO NOTHING;

-- Sample game versions, loaders, categories
-- Game versions is '2'
INSERT INTO loader_field_enum_values (enum_id, value, metadata, created)
VALUES (2, '0.1.0', '{"type":"release","major":false}', '2021-08-18 15:48:58.435729+00');
INSERT INTO loader_field_enum_values (enum_id, value, metadata, created)
VALUES (2, '0.2.0', '{"type":"release","major":false}', '2021-08-18 15:48:59.435729+00');
INSERT INTO loader_field_enum_values (enum_id, value, metadata, created)
VALUES (2, '0.3.0', '{"type":"release","major":false}', '2021-08-18 15:49:00.435729+00');
INSERT INTO loader_field_enum_values (enum_id, value, metadata, created)
VALUES (2, '0.4.0', '{"type":"beta","major":false}', '2021-08-18 15:49:01.435729+00');
INSERT INTO loader_field_enum_values (enum_id, value, metadata, created)
VALUES (2, '0.5.0', '{"type":"release","major":true}', '2061-08-18 15:49:02.435729+00');

-- Also add 'Ordering_Negative1' and 'Ordering_Positive100' to game versions (to test ordering override)
INSERT INTO loader_field_enum_values (enum_id, value, metadata, ordering)
VALUES (2, 'Ordering_Negative1', '{"type":"release","major":false}', -1);
INSERT INTO loader_field_enum_values (enum_id, value, metadata, ordering)
VALUES (2, 'Ordering_Positive100', '{"type":"release","major":false}', 100);

INSERT INTO loader_fields_loaders (loader_id, loader_field_id)
SELECT l.id, lf.id
FROM
	loaders AS l
	CROSS JOIN loader_fields AS lf
WHERE lf.field IN ('game_versions', 'environment')
ON CONFLICT DO NOTHING;

INSERT INTO categories (header, category, project_type)
SELECT 'categories', dummy_categories.category, project_types.id
FROM project_types
CROSS JOIN (
	VALUES
		('combat'),
		('decoration'),
		('economy'),
		('food'),
		('magic'),
		('mobs'),
		('optimization')
) AS dummy_categories(category)
WHERE project_types.name IN ('mod', 'modpack')
	AND NOT EXISTS (
		SELECT 1
		FROM categories
		WHERE categories.project_type = project_types.id
			AND categories.category = dummy_categories.category
	);

-- Create dummy oauth client, secret_hash is SHA512 hash of full lowercase alphabet
INSERT INTO oauth_clients (
        id,
        name,
        icon_url,
        max_scopes,
        secret_hash,
        created_by
    )
VALUES (
        1,
        'oauth_client_alpha',
        NULL,
        {{all_scopes}},
        '4dbff86cc2ca1bae1e16468a05cb9881c97f1753bce3619034898faa1aabe429955a1bf8ec483d7421fe3c1646613a59ed5441fb0f321389f77f48a879c7b1f1',
        3
    );
INSERT INTO oauth_client_redirect_uris (id, client_id, uri)
VALUES (1, 1, 'https://shroudedit.com/oauth_callback');

-- Create dummy data table to mark that this file has been run
CREATE TABLE dummy_data (
	update_id BIGINT PRIMARY KEY
);
