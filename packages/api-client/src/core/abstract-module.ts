import type { AbstractShroudEditClient } from './abstract-client'

export abstract class AbstractModule {
	protected client: AbstractShroudEditClient

	public constructor(client: AbstractShroudEditClient) {
		this.client = client
	}

	/**
	 * Get the module's name, used for error reporting & for module field generation.
	 * @returns Module name
	 */
	public abstract getModuleID(): string
}
