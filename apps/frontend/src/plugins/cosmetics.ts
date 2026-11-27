import type { DarkTheme } from './theme/index.ts'

export type DisplayMode = 'list' | 'gallery' | 'grid'

export type DisplayLocation = 'mod' | 'modpack' | 'schematic' | 'server' | 'user' | 'collection'

export interface Cosmetics {
	rightSearchLayout: boolean
	leftContentLayout: boolean
	advancedRendering: boolean
	externalLinksNewTab: boolean
	notUsingBlockers: boolean
	hideShroudEditAppPromos: boolean
	preferredDarkTheme: DarkTheme
	searchDisplayMode: Record<DisplayLocation, DisplayMode>
	hideDevBanner: boolean
}

export default defineNuxtPlugin({
	name: 'cosmetics',
	setup() {
		const config = useRuntimeConfig()
		const cosmetics = useCookie<Cosmetics>('cosmetics', {
			maxAge: 60 * 60 * 24 * 365 * 10,
			sameSite: 'lax',
			secure: config.public.cookieSecure,
			httpOnly: false,
			path: '/',
			default: () => ({
				rightSearchLayout: false,
				leftContentLayout: false,
				advancedRendering: true,
				externalLinksNewTab: true,
				notUsingBlockers: false,
				hideShroudEditAppPromos: false,
				preferredDarkTheme: 'dark',
				searchDisplayMode: {
					mod: 'list',
					modpack: 'list',
					schematic: 'gallery',
					server: 'list',
					user: 'list',
					collection: 'list',
				},
				hideDevBanner: false,
			}),
		})

		return { provide: { cosmetics } }
	},
})
