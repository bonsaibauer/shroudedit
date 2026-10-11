INSERT INTO games (id, name)
SELECT COALESCE(MAX(id), 0) + 1, 'enshrouded'
FROM games
WHERE NOT EXISTS (SELECT 1 FROM games WHERE name = 'enshrouded');

SELECT setval('project_types_id_seq', (SELECT MAX(id) FROM project_types) + 1, false);

UPDATE project_types
SET name = 'server'
WHERE name = 'minecraft_java_server'
	AND NOT EXISTS (SELECT 1 FROM project_types WHERE name = 'server');

INSERT INTO project_types (name)
VALUES ('schematic'), ('server')
ON CONFLICT (name) DO NOTHING;

INSERT INTO loaders (loader, icon, metadata)
VALUES
	('shroudtopia', '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="9"/><circle cx="12" cy="12" r="5"/><path d="m12 8 2 4-2 4-2-4 2-4Z" fill="currentColor" stroke="none"/></svg>', '{"platform":false}'::jsonb),
	('shroudforge', '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><path d="M13.5 2.5c.7 4.2-2 5.5-3.4 8.1-.7-1.4-1.1-2.7-.8-4.3-3.1 2.4-4.8 5-4.8 8.1A7.5 7.5 0 0 0 12 22a7.5 7.5 0 0 0 7.5-7.6c0-4.2-2.6-7.3-6-11.9Z"/><path d="M12.2 12.1c1.5 2.1 2.4 3.5 2.4 5.1A2.6 2.6 0 0 1 12 19.8a2.6 2.6 0 0 1-2.6-2.6c0-1.5.9-3.1 2.8-5.1Z" fill="currentColor" stroke="none"/></svg>', '{"platform":false}'::jsonb)
ON CONFLICT (loader) DO UPDATE SET icon = EXCLUDED.icon, metadata = EXCLUDED.metadata;

INSERT INTO loaders_project_types (joining_loader_id, joining_project_type_id)
SELECT loaders.id, project_types.id
FROM loaders
CROSS JOIN project_types
WHERE loaders.loader IN ('shroudtopia', 'shroudforge')
	AND project_types.name IN ('mod', 'schematic')
ON CONFLICT DO NOTHING;

INSERT INTO loaders_project_types_games (loader_id, project_type_id, game_id)
SELECT loaders.id, project_types.id, games.id
FROM loaders
CROSS JOIN project_types
CROSS JOIN games
WHERE loaders.loader IN ('shroudtopia', 'shroudforge')
	AND project_types.name IN ('mod', 'schematic')
	AND games.name = 'enshrouded'
ON CONFLICT DO NOTHING;

INSERT INTO loaders_project_types_games (loader_id, project_type_id, game_id)
SELECT loaders.id, project_types.id, games.id
FROM loaders
CROSS JOIN project_types
CROSS JOIN games
WHERE loaders.loader = 'mrpack'
	AND project_types.name = 'modpack'
	AND games.name = 'enshrouded'
ON CONFLICT DO NOTHING;

INSERT INTO loader_fields_loaders (loader_id, loader_field_id)
SELECT loaders.id, loader_fields.id
FROM loaders
CROSS JOIN loader_fields
WHERE loaders.loader IN ('shroudtopia', 'shroudforge')
	AND loader_fields.field IN (
		'game_versions',
		'environment'
	)
ON CONFLICT DO NOTHING;

INSERT INTO loader_fields (field, field_type, optional, min_val, max_val)
VALUES
	('schematic_format_version', 'integer', false, 1, 2147483647),
	('world_editor_version', 'text', false, 1, 64),
	('schematic_width', 'integer', false, 1, 4096),
	('schematic_height', 'integer', false, 1, 4096),
	('schematic_depth', 'integer', false, 1, 4096),
	('schematic_installation', 'text', true, 0, 4096)
ON CONFLICT (field) DO UPDATE SET
	field_type = EXCLUDED.field_type,
	optional = EXCLUDED.optional,
	min_val = EXCLUDED.min_val,
	max_val = EXCLUDED.max_val;

INSERT INTO loader_fields_loaders (loader_id, loader_field_id)
SELECT loaders.id, loader_fields.id
FROM loaders
CROSS JOIN loader_fields
WHERE loaders.loader IN ('shroudtopia', 'shroudforge')
	AND loader_fields.field IN (
		'schematic_format_version',
		'world_editor_version',
		'schematic_width',
		'schematic_height',
		'schematic_depth',
		'schematic_installation'
	)
ON CONFLICT DO NOTHING;

INSERT INTO loader_field_enum_values (enum_id, value, created, metadata)
SELECT loader_field_enums.id, versions.value, versions.created, versions.metadata
FROM loader_field_enums
CROSS JOIN (
	VALUES
		('0.7.0.1', '2024-02-07T00:00:00Z'::timestamptz, '{"type":"release","major":true,"revision":491572}'::jsonb),
		('0.7.0.2', '2024-02-26T00:00:00Z'::timestamptz, '{"type":"release","major":false,"revision":497089}'::jsonb),
		('0.7.1.0', '2024-03-26T00:00:00Z'::timestamptz, '{"type":"release","major":true,"revision":510434}'::jsonb),
		('0.7.1.1', '2024-04-17T00:00:00Z'::timestamptz, '{"type":"release","major":true,"revision":516139}'::jsonb),
		('0.7.2.0', '2024-06-05T00:00:00Z'::timestamptz, '{"type":"release","major":true,"revision":532860}'::jsonb),
		('0.7.2.1', '2024-06-13T00:00:00Z'::timestamptz, '{"type":"release","major":false,"revision":535584}'::jsonb),
		('0.7.3.0', '2024-07-29T00:00:00Z'::timestamptz, '{"type":"release","major":true,"revision":554708}'::jsonb),
		('0.7.4.0', '2024-11-05T00:00:00Z'::timestamptz, '{"type":"release","major":true,"revision":601345}'::jsonb),
		('0.7.4.1', '2024-11-21T00:00:00Z'::timestamptz, '{"type":"release","major":false,"revision":606779}'::jsonb),
		('0.7.4.2', '2024-12-02T00:00:00Z'::timestamptz, '{"type":"release","major":false,"revision":610720}'::jsonb),
		('0.8.0.0', '2025-01-28T00:00:00Z'::timestamptz, '{"type":"release","major":true,"revision":635482}'::jsonb),
		('0.8.0.1', '2025-02-13T00:00:00Z'::timestamptz, '{"type":"release","major":true,"revision":645934}'::jsonb),
		('0.8.1.0', '2025-05-13T00:00:00Z'::timestamptz, '{"type":"release","major":true,"revision":697018}'::jsonb),
		('0.8.1.1', '2025-05-22T00:00:00Z'::timestamptz, '{"type":"release","major":false,"revision":703471}'::jsonb),
		('0.8.1.2', '2025-06-06T00:00:00Z'::timestamptz, '{"type":"release","major":false,"revision":710574}'::jsonb),
		('0.9.0.0', '2025-11-10T00:00:00Z'::timestamptz, '{"type":"release","major":true,"revision":844087}'::jsonb),
		('0.9.0.1', '2025-11-19T00:00:00Z'::timestamptz, '{"type":"release","major":false,"revision":852257}'::jsonb),
		('0.9.0.2', '2025-11-25T00:00:00Z'::timestamptz, '{"type":"release","major":false,"revision":855142}'::jsonb),
		('0.9.0.3', '2025-12-16T00:00:00Z'::timestamptz, '{"type":"release","major":false,"revision":873620}'::jsonb),
		('0.9.0.4', '2026-01-26T00:00:00Z'::timestamptz, '{"type":"release","major":false,"revision":893400}'::jsonb),
		('0.9.1.0', '2026-04-23T00:00:00Z'::timestamptz, '{"type":"release","major":true,"revision":1004637}'::jsonb),
		('0.9.1.1', '2026-05-06T00:00:00Z'::timestamptz, '{"type":"release","major":false,"revision":1018982}'::jsonb),
		('0.9.1.2', '2026-06-29T00:00:00Z'::timestamptz, '{"type":"release","major":false,"revision":1076226}'::jsonb)
) AS versions(value, created, metadata)
WHERE loader_field_enums.enum_name = 'game_versions'
ON CONFLICT (enum_id, value) DO UPDATE SET
	created = EXCLUDED.created,
	metadata = EXCLUDED.metadata;

INSERT INTO loader_field_enum_values (enum_id, value)
SELECT loader_field_enums.id, loaders.loader
FROM loader_field_enums
CROSS JOIN loaders
WHERE loader_field_enums.enum_name = 'mrpack_loaders'
	AND loaders.loader IN ('shroudtopia', 'shroudforge')
ON CONFLICT (enum_id, value) DO NOTHING;

INSERT INTO categories (header, category, project_type)
SELECT 'categories', category, project_types.id
FROM project_types
CROSS JOIN (
	VALUES
		('adventure'),
		('cursed'),
		('decoration'),
		('economy'),
		('equipment'),
		('food'),
		('game-mechanics'),
		('library'),
		('magic'),
		('management'),
		('minigame'),
		('mobs'),
		('optimization'),
		('social'),
		('storage'),
		('technology'),
		('transportation'),
		('utility'),
		('worldgen')
) AS shroudedit_categories(category)
WHERE project_types.name IN ('mod', 'modpack', 'schematic', 'server')
	AND NOT EXISTS (
		SELECT 1
		FROM categories
		WHERE categories.project_type = project_types.id
			AND categories.category = shroudedit_categories.category
	);
