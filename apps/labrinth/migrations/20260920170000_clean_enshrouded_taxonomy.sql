WITH canonical_game AS (
	SELECT id
	FROM games
	WHERE name = 'enshrouded'
	ORDER BY id
	LIMIT 1
), legacy_games AS (
	SELECT id
	FROM games
	WHERE name IN ('minecraft-java', 'minecraft-bedrock', 'enshrouded-java', 'enshrouded-bedrock')
)
INSERT INTO loaders_project_types_games (loader_id, project_type_id, game_id)
SELECT DISTINCT associations.loader_id, associations.project_type_id, canonical_game.id
FROM loaders_project_types_games associations
CROSS JOIN canonical_game
WHERE associations.game_id IN (SELECT id FROM legacy_games)
ON CONFLICT DO NOTHING;

UPDATE loader_fields
SET optional = false,
	min_val = 1,
	max_val = CASE
		WHEN field = 'schematic_format_version' THEN 2147483647
		WHEN field IN ('schematic_width', 'schematic_height', 'schematic_depth') THEN 4096
		ELSE max_val
	END
WHERE field IN (
	'schematic_format_version',
	'world_editor_version',
	'schematic_width',
	'schematic_height',
	'schematic_depth'
);

UPDATE loader_fields
SET optional = true,
	min_val = 0,
	max_val = 4096
WHERE field = 'schematic_installation';

DELETE FROM loaders_project_types_games
WHERE game_id IN (
	SELECT id
	FROM games
	WHERE name IN ('minecraft-java', 'minecraft-bedrock', 'enshrouded-java', 'enshrouded-bedrock')
);

DELETE FROM games
WHERE name IN ('minecraft-java', 'minecraft-bedrock', 'enshrouded-java', 'enshrouded-bedrock');

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

DELETE FROM mods_categories
WHERE joining_category_id IN (
	SELECT categories.id
	FROM categories
	INNER JOIN project_types ON project_types.id = categories.project_type
	WHERE project_types.name = 'server'
		AND categories.header <> 'categories'
);

DELETE FROM categories
WHERE project_type = (SELECT id FROM project_types WHERE name = 'server')
	AND header <> 'categories';

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
WHERE project_types.name = 'server'
	AND NOT EXISTS (
		SELECT 1
		FROM categories
		WHERE categories.project_type = project_types.id
			AND categories.category = shroudedit_categories.category
			AND categories.header = 'categories'
	);
