import { provideShroudEditClient } from '@shroudedit/ui'

import { createShroudEditClient } from '~/helpers/api.ts'

export function setupShroudEditClientProvider(auth: Awaited<ReturnType<typeof useAuth>>) {
	const config = useRuntimeConfig()
	const client = createShroudEditClient(auth, {
		apiBaseUrl: config.public.apiBaseUrl.replace('/v2/', '/'),
		archonBaseUrl: config.public.pyroBaseUrl.replace('/v2/', '/'),
		sharedInstancesBaseUrl: config.public.sharedInstancesBaseUrl,
		commitHash: config.public.hash,
		rateLimitKey: config.rateLimitKey,
	})
	provideShroudEditClient(client)
	return client
}
