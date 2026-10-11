import type { Labrinth } from '@shroudedit/api-client'

export type AccountChoice = {
	id: string
	username: string
	avatarUrl?: string | null
	role?: Labrinth.Users.v2.Role | null
}
