export type FaviconVariant = 'default' | 'settings'
export type FaviconEnvironment = 'local' | 'preview' | 'staging'

const PRODUCTION_FAVICON_HREFS = {
	default: {
		light: '/favicon-light-32x32.png',
		dark: '/favicon-32x32.png',
	},
	settings: {
		light: '/favicon-light-settings-32x32.png',
		dark: '/favicon-settings-32x32.png',
	},
} as const

const FAVICON_MEDIA = [
	'(prefers-color-scheme:no-preference)',
	'(prefers-color-scheme:light)',
	'(prefers-color-scheme:dark)',
] as const

const STAGING_API_PREFIX = 'https://staging-api.shroudedit.com'

export function resolveFaviconEnvironment(
	config: {
		public: {
			production?: boolean
			preview?: boolean
			apiBaseUrl?: string
		}
	},
	hostname: string,
): FaviconEnvironment | null {
	if (!config.public.production || hostname === 'localhost' || hostname === '127.0.0.1') {
		return 'local'
	}

	if (String(config.public.apiBaseUrl).startsWith(STAGING_API_PREFIX)) {
		return 'staging'
	}

	if (config.public.preview) {
		return 'preview'
	}

	return null
}

function getFaviconHrefs(variant: FaviconVariant, _environment: FaviconEnvironment | null) {
	return {
		...PRODUCTION_FAVICON_HREFS[variant],
		type: 'image/png',
		sizes: '32x32',
	}
}

export function getFaviconHeadLinks(
	variant: FaviconVariant = 'default',
	environment: FaviconEnvironment | null = null,
) {
	const icons = getFaviconHrefs(variant, environment)
	const appleTouchIcons = getFaviconHrefs(variant, null)
	const hrefByMedia = {
		'(prefers-color-scheme:no-preference)': icons.light,
		'(prefers-color-scheme:light)': icons.light,
		'(prefers-color-scheme:dark)': icons.dark,
	}
	const appleHrefByMedia = {
		'(prefers-color-scheme:no-preference)': appleTouchIcons.light,
		'(prefers-color-scheme:light)': appleTouchIcons.light,
		'(prefers-color-scheme:dark)': appleTouchIcons.dark,
	}

	return FAVICON_MEDIA.flatMap((media) => [
		{
			rel: 'icon',
			type: icons.type,
			sizes: icons.sizes,
			href: hrefByMedia[media],
			media,
			key: `favicon-${media}`,
		},
		{
			rel: 'apple-touch-icon',
			type: appleTouchIcons.type,
			href: appleHrefByMedia[media],
			media,
			sizes: appleTouchIcons.sizes,
			key: `apple-touch-icon-${media}`,
		},
	])
}
