import type { Theme } from './themes.ts'

interface ThemeSettings {
	preference: Theme | 'system'
	value: Theme
	syncAcrossDevices?: boolean
}

export function useThemeSettings(getDefaultTheme?: () => Theme) {
	getDefaultTheme ??= () => 'dark'

	const config = useRuntimeConfig()
	const $settings = useCookie<ThemeSettings | null>('color-mode', {
		maxAge: 60 * 60 * 24 * 365 * 10,
		sameSite: 'lax',
		secure: config.public.cookieSecure,
		httpOnly: false,
		path: '/',
	})

	const defaults = (): ThemeSettings => ({
		preference: 'system',
		value: getDefaultTheme(),
		syncAcrossDevices: true,
	})
	const readSettings = () =>
		$settings.value && typeof $settings.value === 'object' ? $settings.value : null
	if (!readSettings()) $settings.value = defaults()

	function updateSettings(patch: Partial<ThemeSettings>) {
		$settings.value = { ...(readSettings() ?? defaults()), ...patch }
	}

	return reactive({
		preferred: computed({
			get: () => readSettings()?.preference ?? 'system',
			set: (value) => updateSettings({ preference: value }),
		}),

		active: computed({
			get: () => readSettings()?.value ?? getDefaultTheme(),
			set: (value) => updateSettings({ value }),
		}),

		syncAcrossDevices: computed({
			get: () => readSettings()?.syncAcrossDevices ?? true,
			set: (value) => updateSettings({ syncAcrossDevices: value }),
		}),
	})
}
