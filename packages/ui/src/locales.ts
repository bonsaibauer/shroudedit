import type { LocaleMessages } from './composables/i18n'

export const uiLocaleModules = import.meta.glob<{ default: LocaleMessages }>(
	'./locales/*/index.json',
	{ eager: false },
)

export const metaLocaleModules = import.meta.glob<{ default: LocaleMessages }>(
	'./locales/*/meta.json',
	{ eager: true },
)
