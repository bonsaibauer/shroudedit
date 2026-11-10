import type { LocaleMessages } from './composables/i18n'

export const uiLocaleModulesEager = import.meta.glob<{ default: LocaleMessages }>(
	'./locales/*/index.json',
	{ eager: true },
)
