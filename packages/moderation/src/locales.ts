import type { LocaleMessages } from '@shroudedit/ui'

export const moderationLocaleModules = import.meta.glob<{ default: LocaleMessages }>(
	'./locales/*/index.json',
	{ eager: false },
)
