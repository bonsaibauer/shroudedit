-- Schematic uploads and metadata exclusively use JSON format 8.
UPDATE loader_fields SET min_val = 8, max_val = 8 WHERE field = 'schematic_format_version';
