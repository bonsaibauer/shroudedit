INSERT INTO loader_fields_loaders (loader_id, loader_field_id)
SELECT loaders.id, loader_fields.id
FROM loaders
CROSS JOIN loader_fields
WHERE loaders.loader = 'sfpack'
	AND loader_fields.field IN ('environment', 'modpack_loaders')
ON CONFLICT DO NOTHING;
