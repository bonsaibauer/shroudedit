INSERT INTO loaders_project_types (joining_loader_id, joining_project_type_id)
SELECT loaders.id, project_types.id
FROM loaders
CROSS JOIN project_types
WHERE loaders.loader = 'mrpack' AND project_types.name = 'modpack'
ON CONFLICT DO NOTHING;

ALTER TABLE mods
	ADD COLUMN project_type varchar(64) REFERENCES project_types(name);

UPDATE mods AS m
SET project_type = CASE
	WHEN m.components ? 'enshrouded_server' AND m.components->'enshrouded_server' != 'null'::jsonb THEN 'server'
	WHEN m.components ? 'schematic' AND m.components->'schematic' != 'null'::jsonb THEN 'schematic'
	WHEN m.components ? 'enshrouded_mod' AND m.components->'enshrouded_mod' != 'null'::jsonb THEN 'mod'
	ELSE COALESCE(
		(
			SELECT pt.name
			FROM versions v
			INNER JOIN loaders_versions lv ON lv.version_id = v.id
			INNER JOIN loaders_project_types lpt ON lpt.joining_loader_id = lv.loader_id
			INNER JOIN project_types pt ON pt.id = lpt.joining_project_type_id
			WHERE v.mod_id = m.id AND pt.name != 'schematic'
			ORDER BY (pt.name = 'modpack') DESC, pt.name
			LIMIT 1
		),
		(
			SELECT pt.name
			FROM mods_categories mc
			INNER JOIN categories c ON c.id = mc.joining_category_id
			INNER JOIN project_types pt ON pt.id = c.project_type
			WHERE mc.joining_mod_id = m.id
			ORDER BY (pt.name = 'modpack') DESC, pt.name
			LIMIT 1
		),
		'mod'
	)
END;

ALTER TABLE mods
	ALTER COLUMN project_type SET NOT NULL;

CREATE INDEX mods_project_type_idx ON mods(project_type);
