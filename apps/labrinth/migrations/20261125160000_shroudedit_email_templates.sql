WITH template_routes(notification_type, slug) AS (
	VALUES
		('reset_password', 'reset-password'),
		('verify_email', 'verify-email'),
		('auth_provider_added', 'auth-method-added'),
		('auth_provider_removed', 'auth-method-removed'),
		('two_factor_enabled', 'two-factor-added'),
		('two_factor_removed', 'two-factor-removed'),
		('password_changed', 'password-changed'),
		('password_removed', 'password-removed'),
		('email_changed', 'email-changed'),
		('payment_failed', 'payment-failed'),
		('tax_notification', 'tax-notification'),
		('payout_available', 'payout-available'),
		('pat_created', 'personal-access-token-created'),
		('moderation_message_received', 'moderation-thread-message-received'),
		('report_status_updated', 'report-status-updated'),
		('report_submitted', 'report-submitted'),
		('project_status_approved', 'project-status-approved'),
		('project_status_neutral', 'project-status-updated-neutral'),
		('project_transferred', 'project-transferred'),
		('subscription_credited', 'subscription-credited'),
		('discord_role_creator_club', 'discord-role-creator-club')
)
UPDATE notifications_templates AS template
SET
	body_fetch_url = 'https://shroudedit.com/_internal/templates/email/' || route.slug,
	subject_line = regexp_replace(template.subject_line, 'modrinth', 'ShroudEdit', 'gi'),
	plaintext_fallback = regexp_replace(
			regexp_replace(
				regexp_replace(
					regexp_replace(
						replace(
							replace(
								replace(
									template.plaintext_fallback,
									'https://support.modrinth.com/',
									'support@shroudedit.com'
								),
								'https://support.modrinth.com',
								'support@shroudedit.com'
							),
							'https://modrinth.com',
							'https://shroudedit.com'
						),
						'[[:space:]]*[(]using[[:space:]]*the green chat bubble at the bottom of the page[)]',
						'',
						'gi'
					),
					'(through|via)[[:space:]]+(our[[:space:]]+|the[[:space:]]+)?support portal([[:space:]]+at)?([[:space:]]+support@shroudedit[.]com)?',
					'by emailing support@shroudedit.com',
					'gi'
				),
				'support portal:[[:space:]]*support@shroudedit[.]com',
				'Support email: support@shroudedit.com',
				'gi'
			),
			'modrinth',
			'ShroudEdit',
			'gi'
	)
FROM template_routes AS route
WHERE template.channel = 'email'
	AND template.notification_type = route.notification_type;
