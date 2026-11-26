UPDATE mods
SET project_type = 'mod'
WHERE project_type NOT IN ('mod', 'modpack', 'schematic', 'server');

DELETE FROM mods_categories
WHERE joining_category_id IN (
	SELECT categories.id
	FROM categories
	INNER JOIN project_types ON project_types.id = categories.project_type
	WHERE project_types.name NOT IN ('mod', 'modpack', 'schematic', 'server')
);

DELETE FROM categories
WHERE project_type IN (
	SELECT id
	FROM project_types
	WHERE name NOT IN ('mod', 'modpack', 'schematic', 'server')
);

UPDATE loader_field_enums
SET enum_name = 'modpack_loaders'
WHERE enum_name = 'mrpack_loaders';

UPDATE loader_fields
SET field = 'modpack_loaders'
WHERE field = 'mrpack_loaders';

DELETE FROM loader_field_enum_values
WHERE enum_id = (
	SELECT id
	FROM loader_field_enums
	WHERE enum_name = 'modpack_loaders'
)
	AND value NOT IN ('shroudforge', 'eml', 'shroudtopia');

CREATE TEMP TABLE unsupported_loaders ON COMMIT DROP AS
SELECT id
FROM loaders
WHERE loader NOT IN ('shroudforge', 'eml', 'shroudtopia', 'sfpack');

DELETE FROM loaders_versions
WHERE loader_id IN (SELECT id FROM unsupported_loaders);

DELETE FROM loaders_project_types_games
WHERE loader_id IN (SELECT id FROM unsupported_loaders);

DELETE FROM loader_fields_loaders
WHERE loader_id IN (SELECT id FROM unsupported_loaders);

DELETE FROM loaders_project_types
WHERE joining_loader_id IN (SELECT id FROM unsupported_loaders)
	OR joining_project_type_id IN (
		SELECT id
		FROM project_types
		WHERE name NOT IN ('mod', 'modpack', 'schematic', 'server')
	);

DELETE FROM loaders_project_types_games
WHERE project_type_id IN (
	SELECT id
	FROM project_types
	WHERE name NOT IN ('mod', 'modpack', 'schematic', 'server')
);

DELETE FROM loaders
WHERE id IN (SELECT id FROM unsupported_loaders);

DELETE FROM project_types
WHERE name NOT IN ('mod', 'modpack', 'schematic', 'server');
