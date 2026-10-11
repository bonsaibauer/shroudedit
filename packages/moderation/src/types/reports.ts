import type { Labrinth, SharedInstances } from '@shroudedit/api-client'
import type { Thread, User, Version } from '@shroudedit/utils'

export interface OwnershipTarget {
	id: string
	name: string
	slug: string
	avatar_url?: string
	type: 'user' | 'organization'
}

export interface ExtendedReport extends Labrinth.Reports.v3.Report {
	thread: Thread
	reporter_user: User
	project?: Labrinth.Projects.v2.Project
	user?: User
	version?: Version
	target?: OwnershipTarget
	shared_instance?: SharedInstances.Instances.v1.Instance
}
