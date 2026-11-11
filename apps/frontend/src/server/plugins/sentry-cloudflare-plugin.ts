import { sentryCloudflareNitroPlugin } from '@sentry/nuxt/module/plugins'

declare const __SENTRY_RELEASE__: string
declare const __SENTRY_ENVIRONMENT__: string

export default defineNitroPlugin(
	sentryCloudflareNitroPlugin({
		dsn: 'https://c6bbc8fff23b102ddc634d15a2b18f05@o4512409261965312.ingest.de.sentry.io/4512410112294992',
		tracesSampleRate: 0.0001, // match with wrangler.jsonc
		release: __SENTRY_RELEASE__,
		environment: __SENTRY_ENVIRONMENT__,
	}),
)
