import type { AbstractShroudEditClient } from '@shroudedit/api-client'

import { createContext } from './create-context'

export const [injectShroudEditClient, provideShroudEditClient] = createContext<AbstractShroudEditClient>(
	'root',
	'shroudeditClient',
)
