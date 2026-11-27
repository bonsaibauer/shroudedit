INSERT INTO categories (header, category, project_type)
SELECT 'categories', 'sparkworks', project_types.id
FROM project_types
WHERE project_types.name = 'mod'
	AND NOT EXISTS (
		SELECT 1
		FROM categories
		WHERE categories.project_type = project_types.id
			AND categories.header = 'categories'
			AND categories.category = 'sparkworks'
	);
