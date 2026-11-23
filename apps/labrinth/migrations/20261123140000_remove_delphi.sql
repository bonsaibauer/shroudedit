DROP INDEX IF EXISTS threads_messages_thread_id_created_id;

DROP VIEW IF EXISTS delphi_issue_details_with_statuses;

DROP TABLE IF EXISTS delphi_file_scans;
DROP TABLE IF EXISTS delphi_tech_review_queue;
DROP TABLE IF EXISTS delphi_rule_effects;
DROP TABLE IF EXISTS delphi_global_detail_verdicts;
DROP TABLE IF EXISTS delphi_issue_detail_verdicts;
DROP TABLE IF EXISTS delphi_report_issue_details;
DROP TABLE IF EXISTS delphi_report_issues;
DROP TABLE IF EXISTS delphi_reports;
DROP TABLE IF EXISTS delphi_rule_revisions;
DROP TABLE IF EXISTS delphi_rules;

DROP TYPE IF EXISTS delphi_file_scan_outcome;
DROP TYPE IF EXISTS delphi_report_issue_status;
DROP TYPE IF EXISTS delphi_severity;
