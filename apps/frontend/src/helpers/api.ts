import {
	type AbstractFeature,
	type AuthConfig,
	AuthFeature,
	CircuitBreakerFeature,
	NuxtCircuitBreakerStorage,
	type NuxtClientConfig,
	NuxtShroudEditClient,
	PanelVersionFeature,
	VerboseLoggingFeature,
} from '@shroudedit/api-client'
import type { Ref } from 'vue'

import { useVisitorUserAgent } from '~/composables/visitor-user-agent.ts'
import { readEnv } from '~/helpers/env.ts'
import { getFrontendUserAgent, VISITOR_USER_AGENT_HEADER } from '~/helpers/user-agent.ts'

export function createShroudEditClient(
	auth: Ref<{ token: string | undefined }>,
	config: {
		apiBaseUrl: string
		sharedInstancesBaseUrl: string
		commitHash: string
		rateLimitKey?: string
	},
): NuxtShroudEditClient {
	const visitorUserAgent = useVisitorUserAgent()
	const optionalFeatures = [
		import.meta.dev ? (new VerboseLoggingFeature() as AbstractFeature) : undefined,
	].filter(Boolean) as AbstractFeature[]

	const clientConfig: NuxtClientConfig = {
		labrinthBaseUrl: config.apiBaseUrl,
		sharedInstancesBaseUrl: config.sharedInstancesBaseUrl,
		userAgent: () => (import.meta.server ? getFrontendUserAgent(config.commitHash) : undefined),
		headers: visitorUserAgent ? { [VISITOR_USER_AGENT_HEADER]: visitorUserAgent } : undefined,
		rateLimitKey: config.rateLimitKey || (() => readEnv('RATE_LIMIT_IGNORE_KEY')),
		features: [
			new AuthFeature({
				token: async () => auth.value.token,
			} as AuthConfig),
			new CircuitBreakerFeature({
				storage: new NuxtCircuitBreakerStorage(),
				maxFailures: 3,
				resetTimeout: 30000,
			}),
			new PanelVersionFeature(),
			...optionalFeatures,
		],
	}

	return new NuxtShroudEditClient(clientConfig)
}
