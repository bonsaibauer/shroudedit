INSERT INTO loaders (loader, icon, metadata)
VALUES
	('shroudtopia', '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="9"/><circle cx="12" cy="12" r="5"/><path d="m12 8 2 4-2 4-2-4 2-4Z" fill="currentColor" stroke="none"/></svg>', '{"platform":false}'::jsonb),
	('shroudforge', '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><path d="M13.5 2.5c.7 4.2-2 5.5-3.4 8.1-.7-1.4-1.1-2.7-.8-4.3-3.1 2.4-4.8 5-4.8 8.1A7.5 7.5 0 0 0 12 22a7.5 7.5 0 0 0 7.5-7.6c0-4.2-2.6-7.3-6-11.9Z"/><path d="M12.2 12.1c1.5 2.1 2.4 3.5 2.4 5.1A2.6 2.6 0 0 1 12 19.8a2.6 2.6 0 0 1-2.6-2.6c0-1.5.9-3.1 2.8-5.1Z" fill="currentColor" stroke="none"/></svg>', '{"platform":false}'::jsonb),
	('eml', '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><path d="M5 5.5 9.5 3h5L19 5.5v13L14.5 21h-5L5 18.5v-13Z"/><path d="m8 7 2 2m6-2-2 2m-5 5h6m-6 3h6"/><path d="M12 3v3"/></svg>', '{"platform":false}'::jsonb)
ON CONFLICT (loader) DO UPDATE SET icon = EXCLUDED.icon, metadata = EXCLUDED.metadata;

INSERT INTO loaders_project_types (joining_loader_id, joining_project_type_id)
SELECT loaders.id, project_types.id
FROM loaders
CROSS JOIN project_types
WHERE loaders.loader = 'eml'
	AND project_types.name IN ('mod', 'schematic')
ON CONFLICT DO NOTHING;

INSERT INTO loaders_project_types_games (loader_id, project_type_id, game_id)
SELECT loaders.id, project_types.id, games.id
FROM loaders
CROSS JOIN project_types
CROSS JOIN games
WHERE loaders.loader = 'eml'
	AND project_types.name IN ('mod', 'schematic')
	AND games.name = 'enshrouded'
ON CONFLICT DO NOTHING;

INSERT INTO loader_fields_loaders (loader_id, loader_field_id)
SELECT loaders.id, loader_fields.id
FROM loaders
CROSS JOIN loader_fields
WHERE loaders.loader = 'eml'
	AND loader_fields.field IN (
		'game_versions',
		'environment',
		'schematic_format_version',
		'world_editor_version',
		'schematic_width',
		'schematic_height',
		'schematic_depth',
		'schematic_installation'
	)
ON CONFLICT DO NOTHING;

INSERT INTO loader_field_enum_values (enum_id, value)
SELECT loader_field_enums.id, 'eml'
FROM loader_field_enums
WHERE loader_field_enums.enum_name = 'mrpack_loaders'
ON CONFLICT (enum_id, value) DO NOTHING;
