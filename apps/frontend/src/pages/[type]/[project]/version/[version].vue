<template>
	<div>
		<CreateProjectVersionModal v-if="currentMember" ref="editModal" @save="handleVersionSaved" />
		<ConfirmModal
			v-if="currentMember"
			ref="confirmModal"
			:title="formatMessage(messages.confirmTitle)"
			:description="formatMessage(messages.confirmDescription)"
			:has-to-type="false"
			:proceed-label="formatMessage(messages.proceedDeletion)"
			@proceed="deleteVersion()"
		/>
		<ProjectDownloadModal
			ref="dependencyDownloadModal"
			download-reason="dependency"
			:use-route-hash="false"
			:update-route-selection="false"
			@download="emit('onDownload')"
		/>
		<div class="flex flex-col">
			<BackToParentLink
				:to="`/${project.project_type}/${project.slug ? project.slug : project.id}/versions`"
			>
				{{ formatMessage(messages.allVersions) }}
			</BackToParentLink>
			<template v-if="version">
				<Admonition
					v-if="version.files_missing_attribution?.length"
					type="circle-warning"
					:header="formatMessage(messages.unknownEmbeddedContent)"
					:body="formatMessage(messages.unknownEmbeddedContentDescription)"
					class="mb-4"
				>
					<template #actions>
						<div class="flex">
							<ButtonLink
								type="colored"
								color="orange"
								:to="`/${project.project_type}/${
									project.slug ? project.slug : project.id
								}/settings/permissions`"
							>
								{{ formatMessage(commonProjectSettingsMessages.withheldVersionsWarningResolve) }}
								<RightArrowIcon />
							</ButtonLink>
						</div>
					</template>
				</Admonition>
				<VersionPage
					:version="version"
					:enrichment="enrichment"
					:enrichment-loading="dependenciesLoading"
					:members="members"
					:user-link-creator="(user) => (moderator ? `/user/${user.id}` : undefined)"
					:dependency-link-creator="createDependencyLink"
					class="mb-4"
				>
					<template #headerActions="{ primaryFile, promotedFiles }">
						<ButtonLink
							v-tooltip="
								primaryFile?.url
									? primaryFile.filename + ' (' + formatBytes(primaryFile.size) + ')'
									: formatMessage(messages.noPrimaryFile)
							"
							type="colored"
							color="brand"
							:href="decoratedPrimaryFileUrl"
							:download="primaryFile?.filename"
							:disabled="primaryFile?.url === undefined"
							@click="emit('onDownload')"
						>
							<DownloadIcon aria-hidden="true" />
							{{ formatMessage(commonMessages.downloadButton) }}
						</ButtonLink>
						<ButtonLink
							v-if="
								!!primaryFile?.url &&
								isStaff(auth.user) &&
								modSettings.get(moderationSettings.General.SlicerButtonInVersions)
							"
							v-tooltip="`Open in Slicer`"
							target="_blank"
							:href="`https://slicer.run/?url=${encodeURIComponent(primaryFile?.url)}`"
						>
							<ExternalIcon aria-hidden="true" />
							Slicer
						</ButtonLink>
						<ButtonLink
							v-for="file in promotedFiles.filter(
								(x) =>
									!!x &&
									(x.file_type === 'required-resource-pack' ||
										x.file_type === 'optional-resource-pack'),
							)"
							:key="`promoted-file-${file.hashes.sha1}`"
							v-tooltip="file.filename + ' (' + formatBytes(file.size) + ')'"
							:href="
								createProjectDownloadUrl(file.url, {
									reason: 'dependency',
								})
							"
							:download="primaryFile?.filename"
							:disabled="primaryFile?.url === undefined"
							@click="emit('onDownload')"
						>
							<DownloadIcon aria-hidden="true" />
							<template v-if="file.file_type === 'required-resource-pack'">
								{{ formatMessage(messages.requiredResourcePack) }}
							</template>
							<template v-else-if="file.file_type === 'optional-resource-pack'">
								{{ formatMessage(messages.optionalResourcePack) }}
							</template>
						</ButtonLink>
						<template v-if="currentMember">
							<TeleportOverflowMenu
								:label="formatMessage(messages.edit)"
								:tooltip="formatMessage(messages.edit)"
								:options="[
									{
										id: 'edit-metadata',
										label: formatMessage(messages.editMetadata),
										action: () => handleOpenEditVersionModal(version!.id, project.id, 'metadata'),
									},
									{
										id: 'edit-details',
										label: formatMessage(messages.editDetails),
										action: () =>
											handleOpenEditVersionModal(version!.id, project.id, 'add-details'),
									},
									{
										id: 'edit-files',
										label: formatMessage(messages.editFiles),
										action: () => handleOpenEditVersionModal(version!.id, project.id, 'add-files'),
									},
									{
										id: 'delete',
										label: formatMessage(commonMessages.deleteLabel),
										tone: 'red',
										action: () => confirmModal?.show(),
									},
								]"
							>
								<SettingsIcon aria-hidden="true" />
								<template #edit-metadata>
									<BoxIcon aria-hidden="true" />
									{{ formatMessage(messages.editMetadata) }}
								</template>
								<template #edit-details>
									<InfoIcon aria-hidden="true" />
									{{ formatMessage(messages.editDetails) }}
								</template>
								<template #edit-files>
									<FileIcon aria-hidden="true" />
									{{ formatMessage(messages.editFiles) }}
								</template>
								<template #delete>
									<TrashIcon aria-hidden="true" />
									{{ formatMessage(commonMessages.deleteLabel) }}
								</template>
							</TeleportOverflowMenu>
						</template>
						<TeleportOverflowMenu
							type="outlined"
							:label="formatMessage(commonMessages.moreOptionsButton)"
							:tooltip="formatMessage(commonMessages.moreOptionsButton)"
							:options="[
								{
									id: 'report',
									label: formatMessage(commonMessages.reportButton),
									tone: 'red',
									action: () =>
										auth.user ? reportVersion(version!.id) : navigateTo(signInRouteObj),
								},
								{ type: 'divider', shown: flags.developerMode },
								{
									id: 'copy-id',
									label: formatMessage(commonMessages.copyIdButton),
									action: () => copyToClipboard(version!.id),
									shown: flags.developerMode,
								},
								{
									id: 'copy-permalink',
									label: formatMessage(commonMessages.copyPermalinkButton),
									action: () =>
									copyToClipboard(
										`${config.public.siteUrl}/project/${project.id}/version/${version!.id}`,
										),
									shown: flags.developerMode,
								},
							]"
						>
							<MoreVerticalIcon />
							<template #report>
								<ReportIcon aria-hidden="true" />
								{{ formatMessage(commonMessages.reportButton) }}
							</template>
							<template #copy-link>
								<ReportIcon aria-hidden="true" />
								{{ formatMessage(commonMessages.reportButton) }}
							</template>
							<template #copy-id>
								<ClipboardCopyIcon aria-hidden="true" />
								{{ formatMessage(commonMessages.copyIdButton) }}
							</template>
							<template #copy-permalink>
								<ClipboardCopyIcon aria-hidden="true" />
								{{ formatMessage(commonMessages.copyPermalinkButton) }}
							</template>
						</TeleportOverflowMenu>
					</template>
					<template #supplementaryResourceActions="{ file }">
						<ButtonLink
							:href="decorateDownloadUrl(file.url)"
							:title="`Download ${file.filename}`"
							:download="file.filename"
							tabindex="0"
						>
							<DownloadIcon aria-hidden="true" />
							{{ formatMessage(commonMessages.downloadButton) }}
						</ButtonLink>
						<TeleportOverflowMenu
							type="outlined"
							:label="formatMessage(commonMessages.moreOptionsButton)"
							:tooltip="formatMessage(commonMessages.moreOptionsButton)"
							:options="[
								{
									id: 'copy-sha1',
									label: formatMessage(messages.copySha1),
									action: () => copyFileHash(file, 'sha1'),
								},
								{
									id: 'copy-sha512',
									label: formatMessage(messages.copySha512),
									action: () => copyFileHash(file, 'sha512'),
								},
							]"
						>
							<MoreVerticalIcon aria-hidden="true" />
							<template #copy-sha1>
								<CopyIcon aria-hidden="true" />
								{{ formatMessage(messages.copySha1) }}
							</template>
							<template #copy-sha512>
								<CopyIcon aria-hidden="true" />
								{{ formatMessage(messages.copySha512) }}
							</template>
						</TeleportOverflowMenu>
					</template>
					<template #dependencyActions="{ dependency }">
						<ButtonLink
							v-if="
								createDependencyLink({
									project: dependency.project,
									version: dependency.version ?? getDependencyVersion(dependency.dependency),
								})
							"
							v-tooltip="
								formatMessage(
									(dependency.version ?? getDependencyVersion(dependency.dependency))
										? messages.viewVersion
										: messages.viewProject,
								)
							"
							:to="
								createDependencyLink({
									project: dependency.project,
									version: dependency.version ?? getDependencyVersion(dependency.dependency),
								})
							"
							target="_blank"
							class="!w-9 !rounded-full !px-0"
						>
							<ExternalIcon />
						</ButtonLink>
						<ButtonLink
							v-if="
								(dependency.version ?? getDependencyVersion(dependency.dependency)) &&
								dependency.dependency.dependency_type !== 'incompatible'
							"
							v-tooltip="
								getDependencyPrimaryFileTooltip(
									dependency.version ?? getDependencyVersion(dependency.dependency),
								)
							"
							:href="
								getDependencyDownloadUrl(
									dependency.version ?? getDependencyVersion(dependency.dependency),
								)
							"
							:download="
								getDependencyPrimaryFile(
									dependency.version ?? getDependencyVersion(dependency.dependency),
								)?.filename
							"
							:disabled="
								!getDependencyPrimaryFile(
									dependency.version ?? getDependencyVersion(dependency.dependency),
								)?.url
							"
							class="!w-9 !rounded-full !px-0 !text-brand [&>svg]:!text-brand"
						>
							<DownloadIcon />
						</ButtonLink>
						<IconButton
							v-else-if="dependency.project"
							v-tooltip="formatMessage(messages.downloadProject)"
							:label="formatMessage(messages.downloadProject)"
							class="!text-brand [&>svg]:!text-brand"
							@click="openDependencyDownloadModal(dependency.project, $event)"
						>
							<DownloadIcon />
						</IconButton>
					</template>
				</VersionPage>
				<section
					v-if="isSchematic"
					class="mb-4 flex flex-col gap-4 rounded-2xl border border-solid border-surface-4 bg-surface-2 p-4"
				>
					<h2 class="m-0 text-xl font-semibold text-contrast">
						{{ formatMessage(messages.schematicDetails) }}
					</h2>
					<div class="grid gap-4 lg:grid-cols-[minmax(0,1.4fr)_minmax(18rem,1fr)]">
						<div class="overflow-hidden rounded-xl border border-solid border-surface-4 bg-surface-3">
							<img
								v-if="schematicPreview"
								:src="schematicPreview"
								:alt="formatMessage(messages.schematicPreviewAlt, { title: project.title })"
								class="aspect-video h-full w-full object-cover"
								loading="lazy"
							/>
							<div
								v-else
								class="flex aspect-video items-center justify-center p-6 text-center text-secondary"
							>
								{{ formatMessage(messages.noSchematicPreview) }}
							</div>
						</div>
						<dl class="m-0 grid content-start gap-3 rounded-xl bg-surface-3 p-4">
							<div>
								<dt class="text-sm text-secondary">{{ formatMessage(messages.worldEditorVersion) }}</dt>
								<dd class="m-0 font-semibold text-contrast">
									{{ version.world_editor_version }}
								</dd>
							</div>
							<div>
								<dt class="text-sm text-secondary">{{ formatMessage(messages.schematicDimensions) }}</dt>
								<dd class="m-0 font-semibold text-contrast">{{ schematicDimensions }}</dd>
							</div>
							<div>
								<dt class="text-sm text-secondary">{{ formatMessage(messages.schematicFormatVersion) }}</dt>
								<dd class="m-0 font-semibold text-contrast">
									{{ version.schematic_format_version }}
								</dd>
							</div>
							<div v-if="primaryFile">
								<dt class="text-sm text-secondary">{{ formatMessage(messages.schematicFile) }}</dt>
								<dd class="m-0 font-semibold text-contrast">
									{{ primaryFile.filename }} · {{ formatBytes(primaryFile.size) }}
								</dd>
							</div>
						</dl>
					</div>
					<div v-if="primaryFile" class="rounded-xl bg-surface-3 p-4">
						<div class="mb-2 font-semibold text-contrast">{{ formatMessage(messages.checksum) }}</div>
						<code class="block break-all text-xs">SHA-512: {{ primaryFile.hashes.sha512 }}</code>
					</div>
					<div class="rounded-xl bg-surface-3 p-4">
						<div class="mb-2 font-semibold text-contrast">
							{{ formatMessage(messages.installationInstructions) }}
						</div>
						<p class="m-0 whitespace-pre-wrap">
							{{
								version.schematic_installation ||
								formatMessage(messages.defaultSchematicInstallation)
							}}
						</p>
					</div>
				</section>
			</template>
			<template v-else-if="versionError">
				Uh oh, something went wrong.
				<pre>
					{{ versionError }}
				</pre
				>
			</template>
			<template v-else-if="showVersionSkeleton">
				<div class="flex flex-col gap-4 pb-[30rem]">
					<div
						class="mt-4 flex h-[8rem] w-full animate-pulse items-center justify-center rounded-2xl bg-surface-3"
					></div>
					<hr class="m-0 w-full animate-pulse border-surface-4" />
					<div class="grid gap-4 sm:grid-cols-2">
						<div
							class="flex h-[6rem] w-full animate-pulse items-center justify-center rounded-2xl bg-surface-3"
						></div>
						<div
							class="flex h-[6rem] w-full animate-pulse items-center justify-center rounded-2xl bg-surface-3"
						></div>
					</div>
					<div
						class="flex h-[18rem] w-full animate-pulse items-center justify-center rounded-2xl bg-surface-3"
					></div>
				</div>
			</template>
		</div>
	</div>
</template>
<script setup lang="ts">
import type { Labrinth } from '@shroudedit/api-client'
import {
	BoxIcon,
	ClipboardCopyIcon,
	CopyIcon,
	DownloadIcon,
	ExternalIcon,
	FileIcon,
	InfoIcon,
	MoreVerticalIcon,
	ReportIcon,
	RightArrowIcon,
	SettingsIcon,
	TrashIcon,
} from '@shroudedit/assets'
import { moderationSettings } from '@shroudedit/moderation'
import {
	Admonition,
	BackToParentLink,
	Button,
	ButtonLink,
	commonMessages,
	commonProjectSettingsMessages,
	ConfirmModal,
	defineMessages,
	IconButton,
	injectShroudEditClient,
	injectNotificationManager,
	injectProjectPageContext,
	TeleportOverflowMenu,
	useFormatBytes,
	useFormatDateTime,
	useVIntl,
	VersionPage,
} from '@shroudedit/ui'
import { isStaff } from '@shroudedit/utils'
import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { onServerPrefetch } from 'vue'

import CreateProjectVersionModal from '~/components/ui/create-project-version/CreateProjectVersionModal.vue'
import ProjectDownloadModal from '~/components/ui/ProjectDownloadModal/index.vue'
import { getSignInRouteObj } from '~/composables/auth.ts'
import { projectQueryOptions, STALE_TIME } from '~/composables/queries/project'
import { versionQueryOptions } from '~/composables/queries/version'
import { reportVersion } from '~/utils/report-helpers.ts'

const emit = defineEmits<{
	onDownload: []
}>()

const data = useNuxtApp()
const config = useRuntimeConfig()
const route = useNativeRoute()
const router = useRouter()
const modSettings = useModerationSettings()
const auth = await useAuth()
const tags = useGeneratedState()
const client = injectShroudEditClient()
const queryClient = useQueryClient()
const { addNotification } = injectNotificationManager()
const { createProjectDownloadUrl } = useCdnDownloadContext()
const formatDate = useFormatDateTime({ dateStyle: 'medium' })
const formatBytes = useFormatBytes()
const { formatMessage } = useVIntl()

const {
	projectV2: project,
	projectV3,
	currentMember,
	allMembers: members,
	versions,
	versionsLoading,
	loadVersions,
	dependencies: contextDependencies,
	dependenciesLoading,
	loadDependencies,
	invalidate,
	cdnDownloadReason,
} = injectProjectPageContext()

loadVersions()
loadDependencies()

const flags = useFeatureFlags()

const signInRouteObj = computed(() => getSignInRouteObj(route))

const versionRouteParam = computed(() => route.params.version as string)
const isLatestRoute = computed(() => versionRouteParam.value === 'latest')

function filterVersionsForLatestRoute(allVersions: Labrinth.Versions.v3.Version[]) {
	let filtered = allVersions

	const loaderFilter = route.query.loader
	if (typeof loaderFilter === 'string') {
		filtered = filtered.filter((x) => x.loaders.includes(loaderFilter))
	}

	const gameVersionFilter = route.query.version
	if (typeof gameVersionFilter === 'string') {
		filtered = filtered.filter((x) => x.game_versions.includes(gameVersionFilter))
	}

	return filtered
}

const latestVersionId = computed(() => {
	if (!isLatestRoute.value) {
		return null
	}

	const filtered = filterVersionsForLatestRoute(versions.value ?? [])
	if (filtered.length === 0) return null

	return filtered.reduce((a, b) => (a.date_published > b.date_published ? a : b)).id
})

const versionLookupKey = computed(() =>
	isLatestRoute.value ? latestVersionId.value : versionRouteParam.value,
)

const {
	data: version,
	refetch: refetchVersion,
	error: versionError,
	isPending: versionPending,
} = useQuery({
	queryKey: computed(
		() => ['project', project.value.id, 'version', 'v3', versionLookupKey.value] as const,
	),
	queryFn: () =>
		client.labrinth.versions_v3.getVersionFromIdOrNumber(project.value.id, versionLookupKey.value!),
	enabled: computed(() => !!project.value.id && !!versionLookupKey.value),
	staleTime: STALE_TIME,
})

const showVersionSkeleton = computed(() => import.meta.client && versionPending.value)

onServerPrefetch(async () => {
	if (!project.value.id) return

	let lookupKey = versionRouteParam.value

	if (isLatestRoute.value) {
		loadVersions()
		const versionsData = await queryClient.ensureQueryData(
			projectQueryOptions.versionsV3(project.value.id, client),
		)
		const filtered = filterVersionsForLatestRoute(versionsData ?? [])
		if (filtered.length === 0) return
		lookupKey = filtered.reduce((a, b) => (a.date_published > b.date_published ? a : b)).id
	}

	if (!lookupKey || lookupKey === 'latest') return

	await queryClient.ensureQueryData(
		versionQueryOptions.fromProject(project.value.id, lookupKey, client),
	)
})

watch(
	versionError,
	(error) => {
		if (error) {
			showError({
				fatal: true,
				statusCode: 404,
				message: 'Version not found',
			})
		}
	},
	{ immediate: true },
)

watch(
	[isLatestRoute, latestVersionId, versionsLoading, versions],
	() => {
		if (isLatestRoute.value && !versionsLoading.value && versions.value && !latestVersionId.value) {
			showError({
				fatal: true,
				statusCode: 404,
				message: 'No version matches the filters',
			})
		}
	},
	{ immediate: true },
)

const enrichment = computed(() => contextDependencies.value ?? undefined)

const projectOnlyDependencyProjectIds = computed(() => {
	const projectIds = new Set<string>()
	for (const dependency of version.value?.dependencies ?? []) {
		if (
			dependency.project_id &&
			!dependency.version_id &&
			dependency.dependency_type !== 'incompatible'
		) {
			projectIds.add(dependency.project_id)
		}
	}
	return [...projectIds]
})

const dependencyResolutionLoaders = computed(() => {
	if (!version.value) return []
	if (version.value.loaders.includes('mrpack')) {
		return (version.value.mrpack_loaders ?? []).filter((loader) => loader !== 'enshrouded')
	}
	return version.value.loaders
})

const { data: projectOnlyDependencyVersions } = useQuery({
	queryKey: computed(
		() =>
			[
				'version-page',
				'project-only-dependency-versions',
				version.value?.id,
				version.value?.game_versions ?? [],
				dependencyResolutionLoaders.value,
				projectOnlyDependencyProjectIds.value,
			] as const,
	),
	queryFn: async () => {
		const currentVersion = version.value!
		const entries = await Promise.all(
			projectOnlyDependencyProjectIds.value.map(async (projectId) => {
				let versions: Labrinth.Versions.v2.Version[] = []
				try {
					versions = await client.labrinth.versions_v2.getProjectVersions(projectId, {
						game_versions: currentVersion.game_versions,
						loaders: dependencyResolutionLoaders.value,
						include_changelog: false,
						limit: 100,
					})
				} catch {
					return [projectId, undefined] as const
				}

				return [
					projectId,
					getOnlyCompatibleDependencyVersion(
						versions,
						currentVersion.game_versions,
						dependencyResolutionLoaders.value,
					),
				] as const
			}),
		)

		const versionsByProjectId: Record<string, Labrinth.Versions.v2.Version> = {}
		for (const [projectId, dependencyVersion] of entries) {
			if (dependencyVersion) {
				versionsByProjectId[projectId] = dependencyVersion
			}
		}

		return versionsByProjectId
	},
	enabled: computed(() => !!version.value && projectOnlyDependencyProjectIds.value.length > 0),
	staleTime: STALE_TIME,
})

const primaryFile = computed(
	() => version.value?.files?.find((file) => file.primary) ?? version.value?.files?.[0],
)
const isSchematic = computed(
	() => project.value.project_type === 'schematic' || project.value.actualProjectType === 'schematic',
)
const schematicPreview = computed(
	() =>
		project.value.gallery?.find((image) => image.featured)?.url ??
		project.value.gallery?.[0]?.url,
)
const schematicDimensions = computed(() => {
	if (!version.value) return '—'
	const { schematic_width: width, schematic_height: height, schematic_depth: depth } =
		version.value
	return width && height && depth ? `${width} × ${height} × ${depth}` : '—'
})

const title = computed(() =>
	version.value ? ` ${version.value.version_number} - ${project.value.title}` : undefined,
)

const description = computed(() => {
	if (!version.value) return ''

	return `Download ${project.value.title} ${
		version.value.version_number
	} on ShroudEdit. Supports ${(data as any).$formatVersion(version.value.game_versions)} ${(
		version.value.loaders ?? []
	)
		.map((x: string) => x.charAt(0).toUpperCase() + x.slice(1))
		.join(
			' & ',
		)}. Published on ${formatDate(version.value.date_published)}. ${version.value.downloads} downloads.`
})

useSeoMeta({
	title,
	description,
	ogTitle: title,
	ogDescription: description,
})

const editModal = useTemplateRef('editModal')
const confirmModal = useTemplateRef('confirmModal')
const dependencyDownloadModal = useTemplateRef('dependencyDownloadModal')

async function handleVersionSaved() {
	await Promise.all([
		invalidate(),
		queryClient.invalidateQueries({ queryKey: ['project', project.value.id, 'version', 'v3'] }),
		refetchVersion(),
	])
}

function handleOpenEditVersionModal(versionId: string, projectId: string, stageId: string) {
	if (!currentMember.value) return
	editModal.value?.openEditVersionModal(versionId, projectId, stageId)
}

function openDependencyDownloadModal(
	dependencyProject: Labrinth.Projects.v2.Project,
	event: MouseEvent,
) {
	const baseGameVersions = new Set(version.value?.game_versions ?? [])
	const baseLoaders = new Set(dependencyResolutionLoaders.value)

	dependencyDownloadModal.value?.show(event, {
		projectId: dependencyProject.id,
		incompatibleGameVersions: dependencyProject.game_versions.filter(
			(gameVersion) => !baseGameVersions.has(gameVersion),
		),
		incompatibleLoaders: dependencyProject.loaders.filter((loader) => !baseLoaders.has(loader)),
	})
}

const deleteVersionMutation = useMutation({
	mutationFn: () => client.labrinth.versions_v3.deleteVersion(version.value!.id),
	onSuccess: async () => {
		addNotification({
			title: formatMessage(messages.versionDeletedTitle),
			text: formatMessage(messages.versionDeletedText),
			type: 'success',
		})
		await invalidate()
		await router.replace(`/${project.value.project_type}/${project.value.id}/settings/versions`)
	},
	onError: (err: { data?: { description?: string } }) => {
		addNotification({
			title: formatMessage(commonMessages.errorNotificationTitle),
			text: err.data?.description ?? String(err),
			type: 'error',
		})
	},
})

async function deleteVersion() {
	startLoading()
	try {
		await deleteVersionMutation.mutateAsync()
	} finally {
		stopLoading()
	}
}

const decoratedPrimaryFileUrl = computed(() => {
	const url = primaryFile.value?.url
	if (!url) return undefined
	return createProjectDownloadUrl(url, { reason: cdnDownloadReason.value })
})

function decorateDownloadUrl(url: string) {
	return createProjectDownloadUrl(url, { reason: cdnDownloadReason.value })
}

const messages = defineMessages({
	noPrimaryFile: {
		id: 'version.download.no-primary-file',
		defaultMessage: 'Error: No primary file found',
	},
	versionDeletedTitle: {
		id: 'version.notification.deleted-title',
		defaultMessage: 'Version deleted',
	},
	versionDeletedText: {
		id: 'version.notification.deleted-text',
		defaultMessage: 'The version has been successfully deleted.',
	},
	downloadVersion: {
		id: 'version.download.version',
		defaultMessage: 'Download {version} ({size})',
	},
	edit: {
		id: 'version.edit.button',
		defaultMessage: 'Edit',
	},
	editMetadata: {
		id: 'version.edit.metadata',
		defaultMessage: 'Edit metadata',
	},
	editDetails: {
		id: 'version.edit.details',
		defaultMessage: 'Edit details',
	},
	editFiles: {
		id: 'version.edit.files',
		defaultMessage: 'Edit files',
	},
	copySha1: {
		id: 'version.supplementary-resources.copy-hash-sha1',
		defaultMessage: 'Copy SHA-1',
	},
	copySha512: {
		id: 'version.supplementary-resources.copy-hash-sha512',
		defaultMessage: 'Copy SHA-512',
	},
	allVersions: {
		id: 'version.all-versions',
		defaultMessage: 'All versions',
	},
	schematicDetails: {
		id: 'version.schematic.details',
		defaultMessage: 'Schematic details',
	},
	schematicPreviewAlt: {
		id: 'version.schematic.preview-alt',
		defaultMessage: 'Preview of {title}',
	},
	noSchematicPreview: {
		id: 'version.schematic.no-preview',
		defaultMessage: 'No gallery preview has been published for this schematic.',
	},
	worldEditorVersion: {
		id: 'version.schematic.world-editor-version',
		defaultMessage: 'World Editor version',
	},
	schematicDimensions: {
		id: 'version.schematic.dimensions',
		defaultMessage: 'Saved world area',
	},
	schematicFormatVersion: {
		id: 'version.schematic.format-version',
		defaultMessage: 'Schematic format',
	},
	schematicFile: {
		id: 'version.schematic.file',
		defaultMessage: 'File',
	},
	checksum: {
		id: 'version.schematic.checksum',
		defaultMessage: 'File integrity',
	},
	installationInstructions: {
		id: 'version.schematic.installation',
		defaultMessage: 'Installation',
	},
	defaultSchematicInstallation: {
		id: 'version.schematic.sfbp-installation',
		defaultMessage:
			'Download the .sfbp file, open the ShroudForge World Editor, choose Import blueprint, and select the downloaded file.',
	},
	unknownEmbeddedContent: {
		id: 'version.unknown-embedded-content.title',
		defaultMessage: 'Withheld due to unknown embedded content',
	},
	unknownEmbeddedContentDescription: {
		id: 'version.unknown-embedded-content.description',
		defaultMessage: `This version is currently withheld and not publicly listed. Please provide proof that you have permission to redistribute certain files included.`,
	},
	viewProject: {
		id: 'version.dependency.view-project',
		defaultMessage: `View project`,
	},
	viewVersion: {
		id: 'version.dependency.view-version',
		defaultMessage: `View version`,
	},
	downloadProject: {
		id: 'version.download.download-dependency',
		defaultMessage: 'Select a version to download',
	},
	requiredResourcePack: {
		id: 'version.download.required-resource-pack',
		defaultMessage: 'Required resource pack',
	},
	optionalResourcePack: {
		id: 'version.download.optional-resource-pack',
		defaultMessage: 'Optional resource pack',
	},
	confirmTitle: {
		id: 'version.confirm-delete.title',
		defaultMessage: 'Are you sure you want to delete this version?',
	},
	confirmDescription: {
		id: 'version.confirm-delete.description',
		defaultMessage: 'This version will be permanently deleted. This action cannot be undone.',
	},
	proceedDeletion: {
		id: 'version.confirm-delete.proceed',
		defaultMessage: 'Delete version',
	},
})

const copyFileHash = async (
	file: Labrinth.Versions.v3.VersionFile,
	method: Labrinth.Versions.v3.FileHashType,
) => {
	await copyToClipboard(file.hashes[method])
}

async function copyToClipboard(text: string) {
	await navigator.clipboard.writeText(text)
}

const dependencyVersionPrimaryFiles = computed(() => {
	const versions = [
		...(enrichment.value?.versions ?? []),
		...Object.values(projectOnlyDependencyVersions.value ?? {}),
	]
	const primaryFileMap: Record<string, Labrinth.Versions.v2.VersionFile> = {}
	versions.forEach((depVersion) => {
		const depPrimaryFile = depVersion.files.find((file) => file.primary) ?? depVersion.files[0]

		primaryFileMap[depVersion.id] = depPrimaryFile
	})
	return primaryFileMap
})

function getDependencyVersion(dependency: Labrinth.Versions.v3.Dependency) {
	if (dependency.version_id || !dependency.project_id) {
		return undefined
	}

	return projectOnlyDependencyVersions.value?.[dependency.project_id]
}

function getOnlyCompatibleDependencyVersion(
	versions: Labrinth.Versions.v2.Version[],
	gameVersions: string[],
	loaders: string[],
) {
	const compatibleVersions = versions
		.filter(
			(version) =>
				version.game_versions.some((gameVersion) => gameVersions.includes(gameVersion)) &&
				version.loaders.some((loader) => loaders.includes(loader)),
		)
		.slice()
		.sort((a, b) => new Date(b.date_published).getTime() - new Date(a.date_published).getTime())
	const targetKeys = new Set(
		compatibleVersions.flatMap((version) =>
			version.game_versions.flatMap((gameVersion) =>
				gameVersions.includes(gameVersion)
					? version.loaders
							.filter((loader) => loaders.includes(loader))
							.map((loader) => `${gameVersion}:${loader}`)
					: [],
			),
		),
	)

	return targetKeys.size === 1 ? compatibleVersions[0] : undefined
}

function getDependencyPrimaryFile(version?: Labrinth.Versions.v2.Version) {
	return version ? dependencyVersionPrimaryFiles.value[version.id] : undefined
}

function getDependencyPrimaryFileTooltip(version?: Labrinth.Versions.v2.Version) {
	const dependencyPrimaryFile = getDependencyPrimaryFile(version)
	return dependencyPrimaryFile
		? formatMessage(messages.downloadVersion, {
				version: version?.name || version?.version_number || dependencyPrimaryFile.filename,
				size: formatBytes(dependencyPrimaryFile.size),
			})
		: formatMessage(messages.noPrimaryFile)
}

function getDependencyDownloadUrl(version?: Labrinth.Versions.v2.Version) {
	const dependencyPrimaryFile = getDependencyPrimaryFile(version)
	return dependencyPrimaryFile?.url
		? createProjectDownloadUrl(dependencyPrimaryFile.url, {
				reason: 'dependency',
			})
		: undefined
}

function createDependencyLink(context: {
	project?: Labrinth.Projects.v2.Project
	version?: Labrinth.Versions.v2.Version
}) {
	const baseUrl = context.version
		? `/project/${context.version.project_id}/version/${context.version.id}`
		: context.project
			? `/project/${context.project.id}`
			: undefined
	return baseUrl
		? createProjectDownloadUrl(baseUrl, {
				reason: 'dependency',
			})
		: undefined
}

const moderator = computed(() => isStaff(auth.value?.user))
</script>
