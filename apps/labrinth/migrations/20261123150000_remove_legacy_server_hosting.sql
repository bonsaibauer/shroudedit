DROP TABLE IF EXISTS users_redeemals;

DELETE FROM notifications_templates
WHERE notification_type = 'server_invite';

DELETE FROM users_notifications_preferences
WHERE notification_type = 'server_invite';

DELETE FROM notifications_types
WHERE name = 'server_invite';

DELETE FROM products_tax_identifiers
WHERE tax_processor_id = 'modrinth-servers';
