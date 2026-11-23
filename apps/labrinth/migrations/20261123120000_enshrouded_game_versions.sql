INSERT INTO loader_field_enum_values (enum_id, value, created, metadata)
SELECT loader_field_enums.id, versions.value, versions.created, versions.metadata
FROM loader_field_enums
CROSS JOIN (
	VALUES
		('1.0.0.0', '2026-10-15T15:00:00Z'::timestamptz, '{"type":"release","major":true}'::jsonb),
		('1.0.0.1', '2026-10-15T19:03:00Z'::timestamptz, '{"type":"release","major":false,"revision":1267041}'::jsonb),
		('1.0.0.2', '2026-10-16T00:00:00Z'::timestamptz, '{"type":"release","major":false,"revision":1269579}'::jsonb),
		('1.0.0.3', '2026-10-21T00:00:00Z'::timestamptz, '{"type":"release","major":false,"revision":1273043}'::jsonb),
		('1.0.1.0', '2026-11-02T00:00:00Z'::timestamptz, '{"type":"release","major":false,"revision":1283531}'::jsonb),
		('1.0.1.1', '2026-11-03T00:00:00Z'::timestamptz, '{"type":"release","major":false,"revision":1285601}'::jsonb),
		('1.0.1.2', '2026-11-04T00:00:00Z'::timestamptz, '{"type":"release","major":false,"revision":1287369}'::jsonb)
) AS versions(value, created, metadata)
WHERE loader_field_enums.enum_name = 'game_versions'
ON CONFLICT (enum_id, value) DO UPDATE SET
	created = EXCLUDED.created,
	metadata = EXCLUDED.metadata;
