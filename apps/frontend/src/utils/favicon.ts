export type FaviconVariant = 'default' | 'settings'
export type FaviconEnvironment = 'local' | 'preview' | 'dev'

export function resolveFaviconEnvironment(
	config: {
		public: {
			production?: boolean
			preview?: boolean
		}
	},
	hostname: string,
): FaviconEnvironment | null {
	if (!config.public.production || hostname === 'localhost' || hostname === '127.0.0.1') {
		return 'local'
	}

	if (hostname === 'dev.shroudedit.com') {
		return 'dev'
	}

	if (config.public.preview) {
		return 'preview'
	}

	return null
}

export function getFaviconHeadLinks(
	_variant: FaviconVariant = 'default',
	_environment: FaviconEnvironment | null = null,
) {
	return [
		{
			rel: 'icon',
			type: 'image/x-icon',
			href: '/favicon.ico',
			key: 'favicon',
		},
		{
			rel: 'apple-touch-icon',
			type: 'image/png',
			href: '/apple-touch-icon.png',
			sizes: '180x180',
			key: 'apple-touch-icon',
		},
	]
}
