UPDATE loader_fields
SET min_val = 7,
	max_val = 7
WHERE field = 'schematic_format_version';

UPDATE loader_fields
SET max_val = 65536
WHERE field IN (
	'schematic_width',
	'schematic_height',
	'schematic_depth'
);
