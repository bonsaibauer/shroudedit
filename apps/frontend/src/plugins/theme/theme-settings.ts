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

	function updateSetting<K extends keyof ThemeSettings>(key: K, value: ThemeSettings[K]) {
		const settings = readSettings()
		if (settings?.[key] === value) return
		$settings.value = { ...(settings ?? defaults()), [key]: value }
	}

	return reactive({
		preferred: computed({
			get: () => readSettings()?.preference ?? 'system',
			set: (value) => updateSetting('preference', value),
		}),

		active: computed({
			get: () => readSettings()?.value ?? getDefaultTheme(),
			set: (value) => updateSetting('value', value),
		}),

		syncAcrossDevices: computed({
			get: () => readSettings()?.syncAcrossDevices ?? true,
			set: (value) => updateSetting('syncAcrossDevices', value),
		}),
	})
}
