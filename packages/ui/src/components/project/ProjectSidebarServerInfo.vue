<template>
	<div v-if="loading || hasContent" class="flex flex-col gap-3">
		<h2 class="text-lg m-0">{{ formatMessage(messages.title) }}</h2>

		<div
			v-if="ipAddress"
			v-tooltip="formatMessage(messages.addressTooltip)"
			class="bg-button-bg flex gap-2 justify-between rounded-2xl items-center px-3 pr-1.5 h-12 cursor-pointer hover:bg-button-bg-hover hover:brightness-125 transition-all active:scale-95"
			@click="handleCopyIP"
		>
			<div class="font-semibold truncate">
				{{ ipAddress }}
			</div>
			<div class="w-9 h-9 grid place-content-center">
				<CopyIcon class="shrink-0" />
			</div>
		</div>
		<div
			v-else-if="loading && !projectV3"
			class="h-12 rounded-2xl bg-surface-4 animate-pulse"
		></div>

		<section v-if="requiredContent" class="flex flex-col gap-2">
			<h3 class="text-primary text-base m-0">{{ formatMessage(messages.requiredContent) }}</h3>
			<ServerModpackContentCard
				:name="requiredContent.name"
				:version-number="requiredContent.versionNumber ?? ''"
				:icon="requiredContent.icon"
				:onclick-name="requiredContent.onclickName"
				:onclick-version="requiredContent.onclickVersion"
				:onclick-download="requiredContent.onclickDownload"
				:show-custom-modpack-tooltip="requiredContent.showCustomModpackTooltip"
				:loading-version="loading"
			/>
		</section>
		<section v-else-if="loading" class="flex flex-col gap-2">
			<h3 class="text-primary text-base m-0">{{ formatMessage(messages.requiredContent) }}</h3>
			<div class="h-[52px] rounded-2xl bg-surface-4 animate-pulse"></div>
		</section>
		<section v-if="recommendedVersions.length" class="flex flex-col gap-2">
			<h3 class="text-primary text-base m-0">{{ formatMessage(messages.gameVersion) }}</h3>
			<div class="flex flex-wrap gap-1.5">
				<TagItem
					v-for="version in formatVersionsForDisplay(recommendedVersions, tags.gameVersions)"
					:key="`recommended-tag-${version}`"
				>
					{{ version }}
					<template v-if="supportedVersions.length > 0">
						{{ formatMessage(messages.recommendedVersion) }}
					</template>
				</TagItem>
				<TagItem
					v-for="version in formatVersionsForDisplay(supportedVersionsList, tags.gameVersions)"
					:key="`supported-tag-${version}`"
				>
					{{ version }}
				</TagItem>
				<TagItem
					v-for="loader in loaders ?? []"
					:key="`loader-${loader}`"
					class="border !border-solid border-surface-5"
					:style="`--_color: var(--color-platform-${loader})`"
				>
					<component :is="getLoaderIcon(loader)" v-if="getLoaderIcon(loader)" />
					<FormattedTag :tag="loader" enforce-type="loader" />
				</TagItem>
			</div>
		</section>
		<section v-else-if="loading" class="flex flex-col gap-2">
			<h3 class="text-primary text-base m-0">{{ formatMessage(messages.gameVersion) }}</h3>
			<div class="flex flex-wrap gap-1.5">
				<div
					v-for="width in ['w-16', 'w-20']"
					:key="`version-skeleton-${width}`"
					class="h-[26px] rounded-full bg-surface-4 animate-pulse"
					:class="width"
				></div>
			</div>
		</section>
		<section v-if="props.ping !== undefined || region" class="flex flex-col gap-2">
			<h3 class="text-primary text-base m-0">{{ formatMessage(messages.region) }}</h3>
			<div class="flex flex-wrap gap-1.5 items-center">
				<ServerPing
					v-if="projectV3?.status !== 'draft'"
					:ping="props.ping"
					:status-online="props.statusOnline"
				/>
				<ServerRegion v-if="region" :region="region" />
			</div>
		</section>
		<section v-if="languages.length > 0" class="flex flex-col gap-2">
			<h3 class="text-primary text-base m-0">{{ formatMessage(messages.languages) }}</h3>
			<div class="flex flex-wrap gap-1.5">
				<TagItem v-for="language in languages" :key="`${language}`">
					{{ SERVER_LANGUAGES[language] ? formatMessage(SERVER_LANGUAGES[language]) : language }}
				</TagItem>
			</div>
		</section>
		<section v-if="server" class="flex flex-col gap-2">
			<h3 class="m-0 text-base text-primary">
				{{ formatMessage(messages.accessAndCommunication) }}
			</h3>
			<div class="flex flex-wrap gap-1.5">
				<TagItem>
					{{
						server.text_chat_enabled
							? formatMessage(messages.textChatEnabled)
							: formatMessage(messages.textChatDisabled)
					}}
				</TagItem>
				<TagItem>
					<template v-if="server.voice_chat_enabled">
						{{
							formatMessage(messages.voiceChatEnabled, {
								mode:
									server.voice_chat_mode === 'global'
										? formatMessage(messages.global)
										: formatMessage(messages.proximity),
							})
						}}
					</template>
					<template v-else>{{ formatMessage(messages.voiceChatDisabled) }}</template>
				</TagItem>
			</div>
			<div v-if="server.user_groups.length" class="flex flex-col gap-2">
				<div
					v-for="group in server.user_groups"
					:key="group.name"
					class="rounded-xl border border-solid border-surface-4 bg-surface-2 p-3"
				>
					<div class="flex items-center justify-between gap-2">
						<span class="font-semibold text-contrast">{{ group.name }}</span>
						<button
							v-if="publicPasswordByGroup[group.name]"
							type="button"
							class="cursor-pointer border-0 bg-transparent p-0 text-sm font-semibold text-brand hover:underline"
							@click="copyPassword(group.name)"
						>
							{{ formatMessage(messages.copyPassword) }}
						</button>
						<span v-else class="text-xs text-secondary">{{ passwordStatus(group) }}</span>
					</div>
					<p class="mb-0 mt-1 text-xs text-secondary">
						{{ permissionLabels(group).join(' · ') || formatMessage(messages.noPermissions) }}
					</p>
					<p v-if="group.reserved_slots" class="mb-0 mt-1 text-xs text-secondary">
						{{ formatMessage(messages.reservedSlots, { count: group.reserved_slots }) }}
					</p>
				</div>
			</div>
			<p class="m-0 text-xs text-secondary">{{ formatMessage(messages.ownerProvided) }}</p>
		</section>
	</div>
</template>
<script setup lang="ts">
import type { Labrinth } from '@shroudedit/api-client'
import { CopyIcon, getLoaderIcon } from '@shroudedit/assets'
import { SERVER_LANGUAGES } from '@shroudedit/ui'
import { formatVersionsForDisplay, type GameVersionTag, type PlatformTag } from '@shroudedit/utils'
import { computed } from 'vue'

import { defineMessages, useVIntl } from '../../composables'
import { injectNotificationManager } from '../../providers'
import FormattedTag from '../base/FormattedTag.vue'
import TagItem from '../base/TagItem.vue'
import ServerModpackContentCard from './server/ServerModpackContentCard.vue'
import ServerPing from './server/ServerPing.vue'
import ServerRegion from './server/ServerRegion.vue'

interface RequiredContent {
	name: string
	versionNumber?: string
	icon?: string
	onclickName?: () => void
	onclickVersion?: () => void
	onclickDownload?: () => void
	showCustomModpackTooltip?: boolean
}

interface Props {
	projectV3: Labrinth.Projects.v3.Project | null
	tags: {
		gameVersions: GameVersionTag[]
		loaders: PlatformTag[]
	}
	requiredContent?: RequiredContent | null
	recommendedVersion?: string | null
	supportedVersions?: string[]
	loaders?: string[]
	ping?: number
	statusOnline?: boolean
	loading?: boolean
	publicPasswords?: Labrinth.Projects.v3.PublicServerPassword[]
}

const props = withDefaults(defineProps<Props>(), {
	requiredContent: null,
	recommendedVersion: null,
	supportedVersions: () => [],
	loaders: () => [],
	ping: undefined,
	publicPasswords: () => [],
})

const server = computed(() => props.projectV3?.enshrouded_server ?? null)
const ipAddress = computed(() =>
	server.value?.address ? `${server.value.address}:${server.value.query_port}` : '',
)
const languages = computed(() => server.value?.languages ?? [])
const region = computed(() => server.value?.region)
const publicPasswordByGroup = computed(() =>
	Object.fromEntries(props.publicPasswords.map((entry) => [entry.group_name, entry.password])),
)

const recommendedVersions = computed(() => {
	if (props.recommendedVersion) return [props.recommendedVersion]
	const enshroudedVersion = server.value?.ping?.data?.game_version
	if (enshroudedVersion) return [enshroudedVersion]

	return []
})

const hasContent = computed(
	() =>
		!!ipAddress.value ||
		!!props.requiredContent ||
		recommendedVersions.value.length > 0 ||
		supportedVersionsList.value.length > 0 ||
		languages.value.length > 0 ||
		!!server.value ||
		props.ping !== undefined,
)

const supportedVersionsList = computed(() => {
	if (props.supportedVersions.length > 0) return props.supportedVersions

	return []
})

const { addNotification } = injectNotificationManager()
const { formatMessage } = useVIntl()

function handleCopyIP() {
	navigator.clipboard.writeText(ipAddress.value).then(() => {
		addNotification({
			type: 'success',
			title: formatMessage(messages.copied),
			text: formatMessage(messages.copiedText),
		})
	})
}

function copyPassword(groupName: string) {
	const password = publicPasswordByGroup.value[groupName]
	if (!password) return
	navigator.clipboard.writeText(password).then(() => {
		addNotification({
			type: 'success',
			title: formatMessage(messages.copied),
			text: formatMessage(messages.passwordCopiedText, { group: groupName }),
		})
	})
}

function permissionLabels(group: Labrinth.Projects.v3.EnshroudedUserGroup) {
	return [
		group.can_kick_ban && formatMessage(messages.kickBan),
		group.can_access_inventories && formatMessage(messages.inventories),
		group.can_edit_world && formatMessage(messages.editWorld),
		group.can_edit_base && formatMessage(messages.editBase),
		group.can_extend_base && formatMessage(messages.extendBase),
	].filter((label): label is string => !!label)
}

function passwordStatus(group: Labrinth.Projects.v3.EnshroudedUserGroup) {
	if (group.password_visibility === 'none') return formatMessage(messages.noPassword)
	if (group.password_visibility === 'contact_owner') return formatMessage(messages.contactOwner)
	return formatMessage(messages.passwordRequired)
}

const messages = defineMessages({
	copied: {
		id: `project.about.server.copied`,
		defaultMessage: 'Copied!',
	},
	copiedText: {
		id: `project.about.server.copiedText`,
		defaultMessage: 'Server address copied to clipboard',
	},
	title: {
		id: `project.about.server.title`,
		defaultMessage: 'Server details',
	},
	addressTooltip: {
		id: `project.about.server.address.tooltip`,
		defaultMessage: 'Copy Enshrouded server address and query port',
	},
	requiredContent: {
		id: `project.about.server.requiredContent`,
		defaultMessage: 'Required content',
	},
	gameVersion: {
		id: `project.about.compatibility.game-version`,
		defaultMessage: 'Game version',
	},
	recommendedVersion: {
		id: `project.about.server.recommendedVersion`,
		defaultMessage: '(Recommended)',
	},
	region: {
		id: `project.about.server.region`,
		defaultMessage: 'Region',
	},
	languages: {
		id: `project.about.server.languages`,
		defaultMessage: 'Languages',
	},
	accessAndCommunication: {
		id: `project.about.server.access-and-communication`,
		defaultMessage: 'Access & communication',
	},
	textChatEnabled: {
		id: `project.about.server.text-chat-enabled`,
		defaultMessage: 'Text chat on',
	},
	textChatDisabled: {
		id: `project.about.server.text-chat-disabled`,
		defaultMessage: 'Text chat off',
	},
	voiceChatEnabled: {
		id: `project.about.server.voice-chat-enabled`,
		defaultMessage: 'Voice chat: {mode}',
	},
	voiceChatDisabled: {
		id: `project.about.server.voice-chat-disabled`,
		defaultMessage: 'Voice chat off',
	},
	proximity: { id: `project.about.server.proximity`, defaultMessage: 'Proximity' },
	global: { id: `project.about.server.global`, defaultMessage: 'Global' },
	copyPassword: {
		id: `project.about.server.copy-password`,
		defaultMessage: 'Copy password',
	},
	passwordCopiedText: {
		id: `project.about.server.password-copied`,
		defaultMessage: '{group} password copied to clipboard',
	},
	noPassword: { id: `project.about.server.no-password`, defaultMessage: 'No password' },
	passwordRequired: {
		id: `project.about.server.password-required`,
		defaultMessage: 'Password required',
	},
	contactOwner: {
		id: `project.about.server.contact-owner`,
		defaultMessage: 'Ask owner',
	},
	kickBan: { id: `project.about.server.kick-ban`, defaultMessage: 'Kick/ban' },
	inventories: { id: `project.about.server.inventories`, defaultMessage: 'Inventories' },
	editWorld: { id: `project.about.server.edit-world`, defaultMessage: 'Edit world' },
	editBase: { id: `project.about.server.edit-base`, defaultMessage: 'Edit base' },
	extendBase: { id: `project.about.server.extend-base`, defaultMessage: 'Extend base' },
	noPermissions: {
		id: `project.about.server.no-permissions`,
		defaultMessage: 'Read-only access',
	},
	reservedSlots: {
		id: `project.about.server.reserved-slots`,
		defaultMessage: '{count, plural, one {# reserved slot} other {# reserved slots}}',
	},
	ownerProvided: {
		id: `project.about.server.owner-provided`,
		defaultMessage: 'Settings provided by the server owner.',
	},
})
</script>
