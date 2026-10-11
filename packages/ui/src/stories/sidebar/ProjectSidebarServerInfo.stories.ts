import type { Labrinth } from '@shroudedit/api-client'
import type { GameVersionTag, PlatformTag } from '@shroudedit/utils'
import type { Meta, StoryObj } from '@storybook/vue3-vite'

import ProjectSidebarServerInfo from '../../components/project/ProjectSidebarServerInfo.vue'

const tags = {
	gameVersions: [
		{ version: '0.8.1.0', version_type: 'release', date: '2026-08-18', major: true },
	] as GameVersionTag[],
	loaders: [] as PlatformTag[],
}

const server: Labrinth.Projects.v3.EnshroudedServer = {
	address: 'play.example.com',
	query_port: 15637,
	region: 'europe',
	languages: ['en', 'de'],
	voice_chat_enabled: true,
	voice_chat_mode: 'proximity',
	text_chat_enabled: true,
	user_groups: [
		{
			name: 'Admin',
			role: 'admin',
			can_kick_ban: true,
			can_access_inventories: true,
			can_edit_world: true,
			can_edit_base: true,
			can_extend_base: true,
			reserved_slots: 4,
			password_visibility: 'contact_owner',
		},
		{
			name: 'Friend',
			role: 'friend',
			can_kick_ban: false,
			can_access_inventories: true,
			can_edit_world: true,
			can_edit_base: true,
			can_extend_base: true,
			reserved_slots: 0,
			password_visibility: 'public',
		},
		{
			name: 'Visitor',
			role: 'visitor',
			can_kick_ban: false,
			can_access_inventories: false,
			can_edit_world: false,
			can_edit_base: false,
			can_extend_base: false,
			reserved_slots: 0,
			password_visibility: 'none',
		},
		{
			name: 'Guest',
			role: 'guest',
			can_kick_ban: false,
			can_access_inventories: false,
			can_edit_world: true,
			can_edit_base: false,
			can_extend_base: false,
			reserved_slots: 0,
			password_visibility: 'required',
		},
	],
	ping: {
		when: '2026-09-20T10:00:00Z',
		address: 'play.example.com',
		query_port: 15637,
		data: {
			latency: { nanos: 42000000, secs: 0 },
			name: 'Shroudlands',
			game_version: '0.8.1.0',
			map: 'Main',
			players_online: 5,
			players_max: 16,
			password_protected: true,
		},
	},
}

type Project = Labrinth.Projects.v3.Project

const meta = {
	title: 'Sidebar/ProjectSidebarServerInfo',
	component: ProjectSidebarServerInfo,
	decorators: [
		(story) => ({
			components: { story },
			template: '<div style="max-width: 320px"><story /></div>',
		}),
	],
} satisfies Meta<typeof ProjectSidebarServerInfo>

export default meta
type Story = StoryObj<typeof meta>

export const AccessAndCommunication: Story = {
	args: {
		projectV3: { enshrouded_server: server } as unknown as Project,
		tags,
		publicPasswords: [{ group_name: 'Friend', password: 'friends-only' }],
		ping: 42,
		statusOnline: true,
	},
}

export const OfflineServer: Story = {
	args: {
		projectV3: {
			enshrouded_server: {
				...server,
				address: 'offline.example.com',
				voice_chat_enabled: false,
				text_chat_enabled: false,
				ping: null,
			},
		} as unknown as Project,
		tags,
		ping: 0,
		statusOnline: false,
	},
}
