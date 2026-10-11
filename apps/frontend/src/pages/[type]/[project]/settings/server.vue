<template>
	<div>
		<ConfirmLeaveModal ref="confirmLeaveModal" />
		<section class="universal-card">
			<div class="flex max-w-[600px] flex-col gap-6">
				<h2 class="m-0 text-2xl font-semibold text-contrast">
					{{ formatMessage(messages.heading) }}
				</h2>
				<div class="grid gap-4 sm:grid-cols-[1fr_10rem]">
					<label>
						<span class="label__title">{{ formatMessage(messages.address) }}</span>
						<Input
							v-model="address"
							:placeholder="formatMessage(messages.addressPlaceholder)"
							:disabled="!hasPermission"
							autocomplete="off"
						/>
					</label>
					<label>
						<span class="label__title">{{ formatMessage(messages.queryPort) }}</span>
						<Input
							v-model="queryPort"
							type="number"
							min="1"
							max="65535"
							:disabled="!hasPermission"
						/>
					</label>
				</div>
				<div v-if="address" class="flex items-center gap-2 text-sm">
					<IconButton
						:label="formatMessage(messages.refresh)"
						:disabled="pingLoading"
						type="quiet"
						color="orange"
						size="xs"
						@click="pingServer"
					>
						<SpinnerIcon v-if="pingLoading" class="animate-spin" />
						<RefreshCwIcon v-else />
					</IconButton>
					<span v-if="pingResult?.online" class="text-green">
						{{ formatMessage(messages.online) }}
						<template v-if="pingResult.data?.game_version">
							· {{ pingResult.data.game_version }} · {{ pingResult.data.players_online }}/{{
								pingResult.data.players_max
							}}
						</template>
					</span>
					<span v-else-if="pingResult && !pingLoading" class="text-orange">
						{{ formatMessage(messages.offline) }}
					</span>
				</div>
				<p class="m-0 text-sm text-secondary">{{ formatMessage(messages.portHint) }}</p>
				<label>
					<span class="label__title">{{ formatMessage(messages.region) }}</span>
					<Combobox
						v-model="region"
						:options="regionOptions"
						searchable
						:disabled="!hasPermission"
					/>
				</label>
				<label>
					<span class="label__title">{{ formatMessage(messages.languages) }}</span>
					<MultiSelect
						v-model="languages"
						:options="languageOptions"
						searchable
						include-select-all-option
						:max-tag-rows="2"
						:disabled="!hasPermission"
					/>
				</label>
				<div class="flex flex-col gap-3 border-0 border-t border-solid border-surface-4 pt-5">
					<h3 class="m-0 text-lg text-contrast">{{ formatMessage(messages.communication) }}</h3>
					<div class="flex items-center justify-between gap-4">
						<span>{{ formatMessage(messages.textChat) }}</span>
						<Toggle v-model="textChatEnabled" :disabled="!hasPermission" />
					</div>
					<div class="flex items-center justify-between gap-4">
						<span>{{ formatMessage(messages.voiceChat) }}</span>
						<Toggle v-model="voiceChatEnabled" :disabled="!hasPermission" />
					</div>
					<label v-if="voiceChatEnabled">
						<span class="label__title">{{ formatMessage(messages.voiceChatMode) }}</span>
						<Combobox
							v-model="voiceChatMode"
							:options="voiceChatModeOptions"
							:disabled="!hasPermission"
						/>
					</label>
				</div>

				<div class="flex flex-col gap-3 border-0 border-t border-solid border-surface-4 pt-5">
					<div>
						<h3 class="m-0 text-lg text-contrast">{{ formatMessage(messages.accessGroups) }}</h3>
						<p class="mb-0 mt-1 text-sm text-secondary">
							{{ formatMessage(messages.accessGroupsHint) }}
						</p>
					</div>
					<details
						v-for="group in userGroups"
						:key="group.name"
						class="rounded-xl border border-solid border-surface-4 bg-surface-2 p-3"
					>
						<summary class="cursor-pointer font-semibold text-contrast">
							{{ group.name }}
							<span class="ml-2 text-sm font-normal text-secondary">
								{{ formatGroupSummary(group) }}
							</span>
						</summary>
						<div class="mt-4 flex flex-col gap-3">
							<div class="grid gap-2 sm:grid-cols-2">
								<Checkbox
									v-model="group.can_kick_ban"
									:disabled="!hasPermission"
									:label="formatMessage(messages.kickBan)"
								/>
								<Checkbox
									v-model="group.can_access_inventories"
									:disabled="!hasPermission"
									:label="formatMessage(messages.inventories)"
								/>
								<Checkbox
									v-model="group.can_edit_world"
									:disabled="!hasPermission"
									:label="formatMessage(messages.editWorld)"
								/>
								<Checkbox
									v-model="group.can_edit_base"
									:disabled="!hasPermission"
									:label="formatMessage(messages.editBase)"
								/>
								<Checkbox
									v-model="group.can_extend_base"
									:disabled="!hasPermission"
									:label="formatMessage(messages.extendBase)"
								/>
							</div>
							<label>
								<span class="label__title">{{ formatMessage(messages.reservedSlots) }}</span>
								<Input
									v-model="group.reserved_slots"
									type="number"
									min="0"
									max="16"
									:disabled="!hasPermission"
								/>
							</label>
							<label>
								<span class="label__title">{{ formatMessage(messages.passwordVisibility) }}</span>
								<Combobox
									v-model="group.password_visibility"
									:options="passwordVisibilityOptions(group)"
									:disabled="!hasPermission"
								/>
							</label>
							<label v-if="group.password_visibility === 'public'">
								<span class="label__title">{{ formatMessage(messages.publicPassword) }}</span>
								<Input
									v-model="publicPasswords[group.name]"
									type="password"
									maxlength="128"
									:disabled="!hasPermission"
									autocomplete="new-password"
								/>
								<p v-if="group.role === 'friend'" class="mb-0 mt-1 text-sm text-orange">
									{{ formatMessage(messages.friendWarning) }}
								</p>
							</label>
							<p v-else-if="group.password_visibility === 'none'" class="m-0 text-sm text-orange">
								{{ formatMessage(messages.noPasswordWarning) }}
							</p>
						</div>
					</details>
				</div>
			</div>
		</section>
		<UnsavedChangesPopup
			:original="original"
			:modified="modified"
			:saving="saving"
			@reset="resetChanges"
			@save="handleSave"
		/>
	</div>
</template>

<script setup>
import { RefreshCwIcon, SpinnerIcon } from '@shroudedit/assets'
import {
	Combobox,
	Checkbox,
	commonProjectSettingsMessages,
	ConfirmLeaveModal,
	defineMessages,
	IconButton,
	injectShroudEditClient,
	injectNotificationManager,
	injectProjectPageContext,
	Input,
	MultiSelect,
	SERVER_LANGUAGES,
	SERVER_REGIONS,
	Toggle,
	UnsavedChangesPopup,
	usePageLeaveSafety,
	useVIntl,
} from '@shroudedit/ui'
import { isAdmin } from '@shroudedit/utils'
import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'

const DEFAULT_QUERY_PORT = 15637
const PING_TIMEOUT_MS = 5000
const { formatMessage, locale } = useVIntl()
const messages = defineMessages({
	heading: {
		id: 'project.settings.server.enshrouded-heading',
		defaultMessage: 'Enshrouded server details',
	},
	address: {
		id: 'project.settings.server.enshrouded-address',
		defaultMessage: 'Server address',
	},
	addressPlaceholder: {
		id: 'project.settings.server.enshrouded-address-placeholder',
		defaultMessage: 'play.example.com or 203.0.113.10',
	},
	queryPort: { id: 'project.settings.server.query-port', defaultMessage: 'Query port' },
	portHint: {
		id: 'project.settings.server.query-port-hint',
		defaultMessage:
			'Use the UDP queryPort from enshrouded_server.json. The Enshrouded default is 15637.',
	},
	refresh: { id: 'project.settings.server.refresh-status', defaultMessage: 'Refresh status' },
	online: { id: 'project.settings.server.online', defaultMessage: 'Server is online' },
	offline: {
		id: 'project.settings.server.enshrouded-offline',
		defaultMessage: 'The server did not answer on this UDP query port.',
	},
	region: { id: 'project.settings.server.region', defaultMessage: 'Region' },
	languages: {
		id: 'project.settings.server.languages',
		defaultMessage: 'Community languages',
	},
	communication: { id: 'project.settings.server.communication', defaultMessage: 'Communication' },
	textChat: { id: 'project.settings.server.text-chat', defaultMessage: 'Text chat' },
	voiceChat: { id: 'project.settings.server.voice-chat', defaultMessage: 'Voice chat' },
	voiceChatMode: {
		id: 'project.settings.server.voice-chat-mode',
		defaultMessage: 'Voice chat mode',
	},
	proximity: { id: 'project.settings.server.voice-chat-proximity', defaultMessage: 'Proximity' },
	global: { id: 'project.settings.server.voice-chat-global', defaultMessage: 'Global' },
	accessGroups: { id: 'project.settings.server.access-groups', defaultMessage: 'Access groups' },
	accessGroupsHint: {
		id: 'project.settings.server.access-groups-hint',
		defaultMessage:
			'These settings are provided by the server owner and should match enshrouded_server.json.',
	},
	kickBan: {
		id: 'project.settings.server.permission-kick-ban',
		defaultMessage: 'Kick and ban players',
	},
	inventories: {
		id: 'project.settings.server.permission-inventories',
		defaultMessage: 'Access inventories',
	},
	editWorld: { id: 'project.settings.server.permission-edit-world', defaultMessage: 'Edit world' },
	editBase: { id: 'project.settings.server.permission-edit-base', defaultMessage: 'Edit bases' },
	extendBase: {
		id: 'project.settings.server.permission-extend-base',
		defaultMessage: 'Extend bases',
	},
	reservedSlots: { id: 'project.settings.server.reserved-slots', defaultMessage: 'Reserved slots' },
	passwordVisibility: {
		id: 'project.settings.server.password-visibility',
		defaultMessage: 'Password access',
	},
	passwordNone: { id: 'project.settings.server.password-none', defaultMessage: 'No password' },
	passwordRequired: {
		id: 'project.settings.server.password-required',
		defaultMessage: 'Password required',
	},
	passwordContact: {
		id: 'project.settings.server.password-contact',
		defaultMessage: 'Ask the server owner',
	},
	passwordPublic: {
		id: 'project.settings.server.password-public',
		defaultMessage: 'Show public password',
	},
	publicPassword: {
		id: 'project.settings.server.public-password',
		defaultMessage: 'Public join password',
	},
	friendWarning: {
		id: 'project.settings.server.friend-password-warning',
		defaultMessage:
			'Friend access can modify inventories and bases. Publish this password only if that is intended.',
	},
	publicPasswordMissing: {
		id: 'project.settings.server.public-password-missing',
		defaultMessage: 'Every group configured with a public password needs a password value.',
	},
	accessSaveFailed: {
		id: 'project.settings.server.access-save-failed',
		defaultMessage:
			'The server details were saved, but the public access passwords could not be saved.',
	},
	noPasswordWarning: {
		id: 'project.settings.server.no-password-warning',
		defaultMessage: 'Anyone can join this access group without entering a password.',
	},
	cannotSave: {
		id: 'project.settings.server.enshrouded-cannot-save',
		defaultMessage: 'The Enshrouded server must answer before saving.',
	},
})

const client = injectShroudEditClient()
const queryClient = useQueryClient()
const { addNotification } = injectNotificationManager()
const { projectV3, currentMember, patchProjectV3 } = injectProjectPageContext()
useProjectSettingsHeadTitle(commonProjectSettingsMessages.server)

const address = ref('')
const queryPort = ref(DEFAULT_QUERY_PORT)
const region = ref('')
const languages = ref([])
const voiceChatEnabled = ref(false)
const voiceChatMode = ref('proximity')
const textChatEnabled = ref(false)
const userGroups = ref([])
const publicPasswords = ref({})
const originalState = ref(null)
const initializedProjectId = ref(null)
const pingLoading = ref(false)
const pingResult = ref(null)
const saving = ref(false)
let pingTimer

const hasPermission = computed(() => {
	const EDIT_DETAILS = 1 << 2
	return (
		isAdmin(currentMember.value?.user) ||
		((currentMember.value?.permissions ?? 0) & EDIT_DETAILS) === EDIT_DETAILS
	)
})

async function pingServer() {
	const port = Number(queryPort.value)
	if (!address.value.trim() || !Number.isInteger(port) || port < 1 || port > 65535) {
		pingResult.value = null
		return
	}
	pingLoading.value = true
	try {
		const data = await client.labrinth.server_ping_internal.pingEnshrouded({
			address: address.value.trim(),
			query_port: port,
			timeout_ms: PING_TIMEOUT_MS,
		})
		pingResult.value = { online: true, data }
	} catch {
		pingResult.value = { online: false, data: null }
	} finally {
		pingLoading.value = false
	}
}

watch([address, queryPort], () => {
	clearTimeout(pingTimer)
	pingTimer = setTimeout(pingServer, 500)
})

function defaultUserGroups() {
	return [
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
			password_visibility: 'contact_owner',
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
		{
			name: 'Visitor',
			role: 'visitor',
			can_kick_ban: false,
			can_access_inventories: false,
			can_edit_world: false,
			can_edit_base: false,
			can_extend_base: false,
			reserved_slots: 0,
			password_visibility: 'required',
		},
	]
}

function init(v3, entries = []) {
	if (!v3) return
	const server = v3.enshrouded_server
	address.value = server?.address ?? ''
	queryPort.value = server?.query_port ?? DEFAULT_QUERY_PORT
	region.value = server?.region ?? ''
	languages.value = server?.languages ?? []
	voiceChatEnabled.value = server?.voice_chat_enabled ?? false
	voiceChatMode.value = server?.voice_chat_mode ?? 'proximity'
	textChatEnabled.value = server?.text_chat_enabled ?? false
	userGroups.value = structuredClone(
		server?.user_groups?.length ? server.user_groups : defaultUserGroups(),
	)
	publicPasswords.value = Object.fromEntries(
		entries.map((entry) => [entry.group_name, entry.password]),
	)
	originalState.value = structuredClone(currentState())
	initializedProjectId.value = v3.id
	if (address.value) pingServer()
}

const { data: serverAccessEntries, isFetched: serverAccessFetched } = useQuery({
	queryKey: computed(() => ['project', projectV3.value?.id, 'server-access']),
	queryFn: () => client.labrinth.projects_v3.getServerAccess(projectV3.value.id),
	enabled: computed(() => !!projectV3.value?.id && !!projectV3.value?.enshrouded_server),
})

watch(
	[projectV3, serverAccessEntries, serverAccessFetched],
	([project, entries, accessFetched]) => {
		if (!project || !accessFetched || initializedProjectId.value === project.id) return
		init(project, entries ?? [])
	},
	{ immediate: true },
)

const regionOptions = computed(() =>
	Object.entries(SERVER_REGIONS)
		.sort(([, a], [, b]) => formatMessage(a).localeCompare(formatMessage(b), locale.value))
		.map(([value, name]) => ({ value, label: formatMessage(name) })),
)
const languageOptions = computed(() =>
	Object.entries(SERVER_LANGUAGES)
		.sort(([, a], [, b]) => formatMessage(a).localeCompare(formatMessage(b), locale.value))
		.map(([value, name]) => ({ value, label: formatMessage(name) })),
)
const voiceChatModeOptions = computed(() => [
	{ value: 'proximity', label: formatMessage(messages.proximity) },
	{ value: 'global', label: formatMessage(messages.global) },
])
function passwordVisibilityOptions(group) {
	const options = [
		{ value: 'none', label: formatMessage(messages.passwordNone) },
		{ value: 'required', label: formatMessage(messages.passwordRequired) },
		{ value: 'contact_owner', label: formatMessage(messages.passwordContact) },
	]
	if (group.role !== 'admin' && !group.can_kick_ban) {
		options.push({ value: 'public', label: formatMessage(messages.passwordPublic) })
	}
	return options
}

watch(
	userGroups,
	(groups) => {
		for (const group of groups) {
			if (
				(group.role === 'admin' || group.can_kick_ban) &&
				group.password_visibility === 'public'
			) {
				group.password_visibility = 'contact_owner'
			}
		}
	},
	{ deep: true },
)
function formatGroupSummary(group) {
	const slots = Number(group.reserved_slots) || 0
	return slots > 0 ? `${slots} ${formatMessage(messages.reservedSlots).toLowerCase()}` : ''
}
function currentState() {
	return {
		address: address.value.trim(),
		queryPort: Number(queryPort.value),
		region: region.value,
		languages: languages.value,
		voiceChatEnabled: voiceChatEnabled.value,
		voiceChatMode: voiceChatMode.value,
		textChatEnabled: textChatEnabled.value,
		userGroups: userGroups.value.map((group) => ({
			...group,
			reserved_slots: Number(group.reserved_slots) || 0,
		})),
		publicPasswords: publicPasswords.value,
	}
}

const original = computed(() => originalState.value ?? currentState())
const modified = computed(currentState)
const hasChanges = computed(
	() =>
		initializedProjectId.value !== null &&
		JSON.stringify(original.value) !== JSON.stringify(modified.value),
)
const { confirmLeaveModal } = usePageLeaveSafety(hasChanges)

function resetChanges() {
	if (!originalState.value) return
	const state = structuredClone(originalState.value)
	address.value = state.address
	queryPort.value = state.queryPort
	region.value = state.region
	languages.value = state.languages
	voiceChatEnabled.value = state.voiceChatEnabled
	voiceChatMode.value = state.voiceChatMode
	textChatEnabled.value = state.textChatEnabled
	userGroups.value = state.userGroups
	publicPasswords.value = state.publicPasswords
}

const serverAccessMutation = useMutation({
	mutationFn: ({ projectId, passwords }) =>
		client.labrinth.projects_v3.setServerAccess(projectId, passwords),
	onSuccess: (_data, { projectId, passwords }) => {
		queryClient.setQueryData(['project', projectId, 'server-access'], passwords)
	},
})

async function handleSave() {
	if (!isAdmin(currentMember.value?.user) && !pingResult.value?.online) {
		addNotification({
			title: formatMessage(commonProjectSettingsMessages.server),
			text: formatMessage(messages.cannotSave),
			type: 'error',
		})
		return
	}
	const missingPublicPassword = userGroups.value.some(
		(group) => group.password_visibility === 'public' && !publicPasswords.value[group.name]?.trim(),
	)
	if (missingPublicPassword) {
		addNotification({
			title: formatMessage(commonProjectSettingsMessages.server),
			text: formatMessage(messages.publicPasswordMissing),
			type: 'error',
		})
		return
	}
	saving.value = true
	try {
		const projectSaved = await patchProjectV3({
			enshrouded_server: {
				address: modified.value.address,
				query_port: modified.value.queryPort,
				region: modified.value.region,
				languages: modified.value.languages,
				voice_chat_enabled: modified.value.voiceChatEnabled,
				voice_chat_mode: modified.value.voiceChatMode,
				text_chat_enabled: modified.value.textChatEnabled,
				user_groups: modified.value.userGroups,
			},
		})
		if (!projectSaved) return
		try {
			await serverAccessMutation.mutateAsync({
				projectId: projectV3.value.id,
				passwords: modified.value.userGroups
					.filter((group) => group.password_visibility === 'public')
					.map((group) => ({
						group_name: group.name,
						password: publicPasswords.value[group.name].trim(),
					})),
			})
		} catch {
			addNotification({
				title: formatMessage(commonProjectSettingsMessages.server),
				text: formatMessage(messages.accessSaveFailed),
				type: 'error',
			})
			return
		}
		originalState.value = structuredClone(modified.value)
	} finally {
		saving.value = false
	}
}
</script>
