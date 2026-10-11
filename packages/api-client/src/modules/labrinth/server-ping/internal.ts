import { AbstractModule } from '../../../core/abstract-module'
import type { Labrinth } from '../types'

export class LabrinthServerPingInternalModule extends AbstractModule {
	public getModuleID(): string {
		return 'labrinth_server_ping_internal'
	}

	/** Ping an Enshrouded dedicated server through its UDP query port. */
	public async pingEnshrouded(
		request: Labrinth.ServerPing.Internal.EnshroudedPingRequest,
	): Promise<Labrinth.Projects.v3.EnshroudedServerPingData> {
		return this.client.request<Labrinth.Projects.v3.EnshroudedServerPingData>(
			'/server-ping/enshrouded',
			{
				api: 'labrinth',
				version: 'internal',
				method: 'POST',
				body: request,
			},
		)
	}
}
