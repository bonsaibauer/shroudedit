WITH schematic_categories AS (
	SELECT categories.id
	FROM categories
	INNER JOIN project_types ON project_types.id = categories.project_type
	WHERE project_types.name = 'schematic'
		AND categories.header = 'categories'
)
DELETE FROM mods_categories
WHERE joining_category_id IN (SELECT id FROM schematic_categories);

DELETE FROM categories
USING project_types
WHERE categories.project_type = project_types.id
	AND project_types.name = 'schematic'
	AND categories.header = 'categories';

INSERT INTO categories (header, category, project_type)
SELECT 'categories', schematic_categories.category, project_types.id
FROM project_types
CROSS JOIN (VALUES ('castle'), ('home')) AS schematic_categories(category)
WHERE project_types.name = 'schematic';
