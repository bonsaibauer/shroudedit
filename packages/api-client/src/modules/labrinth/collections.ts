import { AbstractModule } from '../../core/abstract-module.js'
import type { Labrinth } from '../types'

export class LabrinthCollectionsModule extends AbstractModule {
	public getModuleID(): string {
		return 'labrinth_collections'
	}

	/**
	 * Get a collection by ID (v3)
	 *
	 * @param id - Collection ID
	 * @returns Promise resolving to the collection data
	 *
	 * @example
	 * ```typescript
	 * const collection = await client.labrinth.collections.get('c1D2e3F4')
	 * ```
	 */
	public async get(id: string): Promise<Labrinth.Collections.Collection> {
		return this.client.request<Labrinth.Collections.Collection>(`/collection/${id}`, {
			api: 'labrinth',
			version: 3,
			method: 'GET',
		})
	}

	/**
	 * Get multiple collections by IDs (v3)
	 *
	 * @param ids - Array of collection IDs
	 * @returns Promise resolving to array of collections
	 *
	 * @example
	 * ```typescript
	 * const collections = await client.labrinth.collections.getMultiple(['c1D2e3F4', 'c5D6e7F8'])
	 * ```
	 */
	public async getMultiple(ids: string[]): Promise<Labrinth.Collections.Collection[]> {
		return this.client.request<Labrinth.Collections.Collection[]>(`/collections`, {
			api: 'labrinth',
			version: 3,
			method: 'GET',
			params: { ids: JSON.stringify(ids) },
		})
	}

	/**
	 * Edit a collection (v3)
	 *
	 * @param id - Collection ID
	 * @param data - Collection update data
	 *
	 * @example
	 * ```typescript
	 * await client.labrinth.collections.edit('c1D2e3F4', {
	 *   name: 'Updated name',
	 *   description: 'Updated description',
	 *   status: 'listed'
	 * })
	 * ```
	 */
	public async edit(id: string, data: Labrinth.Collections.EditCollectionRequest): Promise<void> {
		return this.client.request(`/collection/${id}`, {
			api: 'labrinth',
			version: 3,
			method: 'PATCH',
			body: data,
		})
	}

	/**
	 * Delete a collection (v3)
	 *
	 * @param id - Collection ID
	 *
	 * @example
	 * ```typescript
	 * await client.labrinth.collections.delete('c1D2e3F4')
	 * ```
	 */
	public async delete(id: string): Promise<void> {
		return this.client.request(`/collection/${id}`, {
			api: 'labrinth',
			version: 3,
			method: 'DELETE',
		})
	}

	/**
	 * Edit a collection icon (v3)
	 *
	 * @param id - Collection ID
	 * @param icon - Icon file
	 * @param ext - File extension (e.g., 'png', 'jpg')
	 *
	 * @example
	 * ```typescript
	 * await client.labrinth.collections.editIcon('c1D2e3F4', iconFile, 'png')
	 * ```
	 */
	public async editIcon(id: string, icon: Blob, ext: string): Promise<void> {
		return this.client.request(`/collection/${id}/icon?ext=${ext}`, {
			api: 'labrinth',
			version: 3,
			method: 'PATCH',
			body: icon,
		})
	}

	/**
	 * Delete a collection icon (v3)
	 *
	 * @param id - Collection ID
	 *
	 * @example
	 * ```typescript
	 * await client.labrinth.collections.deleteIcon('c1D2e3F4')
	 * ```
	 */
	public async deleteIcon(id: string): Promise<void> {
		return this.client.request(`/collection/${id}/icon`, {
			api: 'labrinth',
			version: 3,
			method: 'DELETE',
		})
	}
}
