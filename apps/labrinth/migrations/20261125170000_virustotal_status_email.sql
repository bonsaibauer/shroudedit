INSERT INTO notifications_types
	(name, delivery_priority, expose_in_user_preferences, expose_in_site_notifications)
VALUES
	('virustotal_status_updated', 2, FALSE, FALSE),
	('moderation_information_requested', 2, FALSE, FALSE);

INSERT INTO users_notifications_preferences (user_id, channel, notification_type, enabled)
VALUES
	(NULL, 'email', 'virustotal_status_updated', TRUE),
	(NULL, 'email', 'moderation_information_requested', TRUE);

INSERT INTO notifications_templates
	(channel, notification_type, subject_line, body_fetch_url, plaintext_fallback)
VALUES (
	'email',
	'virustotal_status_updated',
	'{virustotal.subject}',
	'https://shroudedit.com/_internal/templates/email/virustotal-status-updated',
	CONCAT(
		'Hi {user.name},',
		CHR(10),
		CHR(10),
		'{virustotal.message}',
		CHR(10),
		CHR(10),
		'{virustotal.details}',
		CHR(10),
		CHR(10),
		'{virustotal.action_label}: {virustotal.action_url}',
		CHR(10),
		CHR(10),
		'Questions? Contact support@shroudedit.com.'
	)
),
(
	'email',
	'moderation_information_requested',
	'We need a little more information about {project.name}',
	'https://shroudedit.com/_internal/templates/email/moderation-information-requested',
	CONCAT(
		'Hi {user.name},',
		CHR(10),
		CHR(10),
		'While reviewing version {version.name}, our moderation team found something that needs clarification before the review can continue.',
		CHR(10),
		CHR(10),
		'Please sign in and reply to the question in your moderation thread. Downloads will remain unavailable until the review is complete.',
		CHR(10),
		CHR(10),
		'Open moderation thread: https://shroudedit.com/project/{project.id}/moderation',
		CHR(10),
		CHR(10),
		'Questions? Contact support@shroudedit.com.'
	)
);
