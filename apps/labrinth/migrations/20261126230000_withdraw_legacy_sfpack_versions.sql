CREATE TABLE legacy_sfpack_reupload_queue (
	version_id BIGINT PRIMARY KEY REFERENCES versions(id) ON DELETE CASCADE,
	project_id BIGINT NOT NULL REFERENCES mods(id) ON DELETE CASCADE,
	previous_status VARCHAR(32) NOT NULL,
	recorded_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

INSERT INTO legacy_sfpack_reupload_queue (version_id, project_id, previous_status)
SELECT v.id, v.mod_id, v.status
FROM versions v
WHERE NOT (v.components ? 'modpack_contents')
	AND EXISTS (
		SELECT 1
		FROM files f
		WHERE f.version_id = v.id
			AND f.is_primary
			AND LOWER(f.filename) LIKE '%.sfpack'
	);

UPDATE versions v
SET status = 'draft'
FROM legacy_sfpack_reupload_queue q
WHERE v.id = q.version_id
	AND v.status IN ('listed', 'unlisted', 'archived', 'scheduled');
