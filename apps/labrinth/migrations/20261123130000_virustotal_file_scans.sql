CREATE TYPE virustotal_scan_status AS ENUM (
	'queued',
	'submitted',
	'completed',
	'failed',
	'unknown'
);

CREATE TYPE virustotal_finding_verdict AS ENUM ('safe', 'unsafe');

CREATE TABLE virustotal_file_scans (
	file_id BIGINT PRIMARY KEY REFERENCES files(id) ON DELETE CASCADE,
	sha256 TEXT CHECK (sha256 IS NULL OR sha256 ~ '^[0-9a-f]{64}$'),
	analysis_id TEXT,
	status virustotal_scan_status NOT NULL DEFAULT 'queued',
	analysis JSONB,
	error TEXT,
	submitted_at TIMESTAMPTZ,
	checked_at TIMESTAMPTZ,
	next_poll_at TIMESTAMPTZ,
	event_id UUID NOT NULL,
	submission_attempts SMALLINT NOT NULL DEFAULT 0,
	poll_attempts SMALLINT NOT NULL DEFAULT 0,
	updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
	CONSTRAINT virustotal_file_scans_analysis_consistency CHECK (
		(status NOT IN ('submitted', 'completed', 'unknown') OR analysis_id IS NOT NULL)
	)
);

CREATE INDEX virustotal_file_scans_polling
	ON virustotal_file_scans (next_poll_at)
	WHERE status = 'submitted';

CREATE INDEX virustotal_file_scans_sha256 ON virustotal_file_scans (sha256);

CREATE TABLE virustotal_finding_verdicts (
	id BIGSERIAL PRIMARY KEY,
	scope TEXT NOT NULL CHECK (scope IN ('local', 'global')),
	file_id BIGINT REFERENCES files(id) ON DELETE CASCADE,
	sha256 TEXT NOT NULL CHECK (sha256 ~ '^[0-9a-f]{64}$'),
	engine_name TEXT NOT NULL,
	category TEXT NOT NULL,
	result TEXT NOT NULL,
	verdict virustotal_finding_verdict NOT NULL,
	moderator_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
	updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
	CONSTRAINT virustotal_finding_verdicts_scope_consistency CHECK (
		(scope = 'local' AND file_id IS NOT NULL)
		OR (scope = 'global' AND file_id IS NULL)
	)
);

CREATE UNIQUE INDEX virustotal_finding_verdicts_local_unique
	ON virustotal_finding_verdicts (file_id, engine_name, category, result)
	WHERE scope = 'local';

CREATE UNIQUE INDEX virustotal_finding_verdicts_global_unique
	ON virustotal_finding_verdicts (sha256, engine_name, category, result)
	WHERE scope = 'global';

CREATE TABLE virustotal_api_usage (
	id BOOLEAN PRIMARY KEY DEFAULT TRUE CHECK (id),
	minute_started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
	minute_requests INTEGER NOT NULL DEFAULT 0,
	day_started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
	day_requests INTEGER NOT NULL DEFAULT 0
);

INSERT INTO virustotal_api_usage (id) VALUES (TRUE);

CREATE TABLE technical_review_queue (
	project_id BIGINT PRIMARY KEY REFERENCES mods(id) ON DELETE CASCADE
);
