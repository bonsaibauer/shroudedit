INSERT INTO loaders (loader, icon, metadata)
VALUES (
	'sfpack',
	'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><path d="M4 7 12 3l8 4v10l-8 4-8-4V7Z"/><path d="m4 7 8 4 8-4M12 11v10"/></svg>',
	'{"platform":false}'::jsonb
)
ON CONFLICT (loader) DO UPDATE SET icon = EXCLUDED.icon, metadata = EXCLUDED.metadata;

INSERT INTO loaders_project_types (joining_loader_id, joining_project_type_id)
SELECT loaders.id, project_types.id
FROM loaders CROSS JOIN project_types
WHERE loaders.loader = 'sfpack' AND project_types.name = 'modpack'
ON CONFLICT DO NOTHING;

INSERT INTO loaders_project_types_games (loader_id, project_type_id, game_id)
SELECT loaders.id, project_types.id, games.id
FROM loaders CROSS JOIN project_types CROSS JOIN games
WHERE loaders.loader = 'sfpack' AND project_types.name = 'modpack' AND games.name = 'enshrouded'
ON CONFLICT DO NOTHING;

INSERT INTO loader_fields_loaders (loader_id, loader_field_id)
SELECT loaders.id, loader_fields.id
FROM loaders CROSS JOIN loader_fields
WHERE loaders.loader = 'sfpack' AND loader_fields.field = 'game_versions'
ON CONFLICT DO NOTHING;

DELETE FROM loaders_project_types_games
WHERE loader_id = (SELECT id FROM loaders WHERE loader = 'mrpack')
	AND project_type_id = (SELECT id FROM project_types WHERE name = 'modpack')
	AND game_id = (SELECT id FROM games WHERE name = 'enshrouded');

DELETE FROM loaders_project_types
WHERE joining_loader_id = (SELECT id FROM loaders WHERE loader = 'mrpack')
	AND joining_project_type_id = (SELECT id FROM project_types WHERE name = 'modpack');
