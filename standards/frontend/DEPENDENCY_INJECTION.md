# Dependency Injection

The Vue UI uses `provide` and `inject` for state or services shared by distant components. Create typed provider pairs with `createContext` from `packages/ui/src/providers/create-context.ts`.

```ts
import { createContext } from '@shroudedit/ui'
import type { Ref } from 'vue'

interface PageContext {
	selectedProjectId: Ref<string | null>
	refresh: () => Promise<void>
}

export const [injectPageContext, providePageContext] = createContext<PageContext>('ProjectPage')
```

Provide the context from a page or layout, then inject it from descendants that need the shared state:

```ts
providePageContext({ selectedProjectId, refresh })
const page = injectPageContext()
```

Use props and emits for direct parent-child communication. Use a provider when several nested components need the same page state or service. Keep API access behind the ShroudEdit API client provider in `packages/ui/src/providers/api-client.ts`; the website initializes that client in `apps/frontend/src/providers/setup/modrinth-client.ts`.
