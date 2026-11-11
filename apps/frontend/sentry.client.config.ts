import { useRuntimeConfig } from '#imports'
import * as Sentry from '@sentry/nuxt'

const runtimeConfig = useRuntimeConfig()

Sentry.init({
	dsn: runtimeConfig.public.sentry.dsn,
	tracesSampleRate: 0.0001,
	environment: runtimeConfig.public.buildEnv ?? 'development',
	release: runtimeConfig.public.hash === 'unknown' ? undefined : runtimeConfig.public.hash,
})
