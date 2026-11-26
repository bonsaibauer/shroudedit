<template>
	<Tabs
		v-if="editingVersion"
		value="add-details"
		:tabs="editTabs"
		class="mb-5 border border-solid border-surface-5 !shadow-none !drop-shadow-none"
		@change="setEditTab"
	/>
	<div class="flex w-full flex-col gap-6">
		<div
			v-if="packageIdMismatch || packageDifferences.length"
			class="rounded-xl border border-solid border-surface-5 bg-surface-2 p-4"
		>
			<div class="font-semibold text-contrast">{{ formatMessage(messages.packageMismatch) }}</div>
			<p v-if="packageIdMismatch" class="mt-2 text-sm text-secondary">
				{{ formatMessage(messages.packageIdMismatch, {
					packageId: inferredVersionData?.mod_id,
					projectSlug: projectV2.slug,
				}) }}
			</p>
			<div v-for="difference in packageDifferences" :key="difference.key" class="mt-3 text-sm">
				<div class="font-semibold text-contrast">{{ difference.label }}</div>
				<div class="text-secondary">{{ formatMessage(messages.packageValue) }}: {{ difference.packageValue }}</div>
				<div class="text-secondary">{{ formatMessage(messages.catalogValue) }}: {{ difference.catalogValue }}</div>
				<Button type="quiet" class="mt-1" @click="usePackageValue(difference.key)">
					{{ formatMessage(messages.usePackageValue) }}
				</Button>
			</div>
			<p class="mt-2 text-sm text-secondary">{{ formatMessage(messages.updatePackageHint) }}</p>
		</div>
		<div class="flex flex-col gap-2">
			<span class="font-semibold text-contrast">
				{{ formatMessage(messages.versionType) }} <span class="text-red">*</span>
			</span>
			<Chips
				v-model="draftVersion.version_type"
				:items="['release', 'beta', 'alpha']"
				:never-empty="true"
				:capitalize="true"
				:disabled="isUploading"
				hide-checkmark-icon
			/>
		</div>
		<div class="flex flex-col gap-2">
			<span class="font-semibold text-contrast">
				{{ formatMessage(messages.versionNumber) }} <span class="text-red">*</span>
			</span>
			<Input
				id="version-number"
				v-model="draftVersion.version_number"
				:disabled="isUploading"
				:placeholder="formatMessage(messages.versionNumberPlaceholder)"
				autocomplete="off"
				:maxlength="32"
			/>
			<span>{{ formatMessage(messages.versionNumberDescription) }}</span>
		</div>
		<div class="flex flex-col gap-2">
			<span class="font-semibold text-contrast">{{ formatMessage(messages.versionSubtitle) }}</span>
			<Input
				id="version-number"
				v-model="draftVersion.name"
				:placeholder="formatMessage(messages.versionSubtitlePlaceholder)"
				autocomplete="off"
				:maxlength="120"
				:disabled="isUploading"
			/>
		</div>
		<div v-if="projectType === 'schematic'" class="flex flex-col gap-5">
			<div class="rounded-xl border border-solid border-surface-5 bg-surface-2 p-4">
				<div class="font-semibold text-contrast">
					{{ formatMessage(messages.schematicMetadata) }}
				</div>
				<div class="mt-1 text-sm text-secondary">
					{{ formatMessage(messages.schematicMetadataDescription) }}
				</div>
			</div>
			<div class="grid gap-4 sm:grid-cols-2">
				<div class="flex flex-col gap-2">
					<span class="font-semibold text-contrast">
						{{ formatMessage(messages.schematicFormatVersion) }} <span class="text-red">*</span>
					</span>
					<Input
						id="schematic-format-version"
						v-model="draftVersion.schematic_format_version"
						type="number"
						:min="8"
						:max="8"
						:disabled="true"
					/>
				</div>
				<div class="flex flex-col gap-2">
					<span class="font-semibold text-contrast">
						{{ formatMessage(messages.worldEditorVersion) }} <span class="text-red">*</span>
					</span>
					<Input
						id="world-editor-version"
						v-model="draftVersion.world_editor_version"
						:disabled="isUploading"
						:placeholder="formatMessage(messages.worldEditorVersionPlaceholder)"
						:maxlength="64"
					/>
				</div>
			</div>
			<div class="grid gap-4 sm:grid-cols-3">
				<div v-for="dimension in dimensions" :key="dimension.key" class="flex flex-col gap-2">
					<span class="font-semibold text-contrast">
						{{ formatMessage(dimension.label) }} <span class="text-red">*</span>
					</span>
					<Input
						:id="`schematic-${dimension.key}`"
						v-model="draftVersion[dimension.key]"
						type="number"
						:min="1"
						:max="65536"
						:disabled="isUploading"
					/>
				</div>
			</div>
			<div class="flex flex-col gap-2">
				<span class="font-semibold text-contrast">{{ formatMessage(messages.installation) }}</span>
				<Textarea
					id="schematic-installation"
					v-model="draftVersion.schematic_installation"
					:disabled="isUploading"
					:placeholder="formatMessage(messages.installationPlaceholder)"
					:maxlength="4096"
				/>
				<span>{{ formatMessage(messages.previewDescription) }}</span>
			</div>
		</div>
		<div class="flex flex-col gap-2">
			<span class="font-semibold text-contrast">{{
				formatMessage(messages.versionChangelog)
			}}</span>

			<div class="w-full">
				<MarkdownEditor
					v-model="draftVersion.changelog"
					:on-image-upload="onImageUpload"
					:min-height="150"
					:disabled="isUploading"
				/>
			</div>
		</div>
	</div>
</template>

<script lang="ts" setup>
import {
	Button,
	Chips,
	defineMessages,
	Input,
	injectProjectPageContext,
	MarkdownEditor,
	Tabs,
	type TabsTab,
	Textarea,
	useVIntl,
} from '@shroudedit/ui'

import { useImageUpload } from '~/composables/image-upload.ts'
import { injectManageVersionContext } from '~/providers/version/manage-version-modal'

const { draftVersion, inferredVersionData, isUploading, editingVersion, modal, projectType } =
	injectManageVersionContext()
const { projectV2 } = injectProjectPageContext()
const { formatMessage } = useVIntl()

const messages = defineMessages({
	packageMismatch: {
		id: 'create-project-version.package-mismatch',
		defaultMessage: 'Package and publication differ',
	},
	packageIdMismatch: {
		id: 'create-project-version.package-id-mismatch',
		defaultMessage: 'Package mod ID: {packageId}. Project slug: {projectSlug}. Update mod.json or the project slug before uploading.',
	},
	packageValue: {
		id: 'create-project-version.package-value',
		defaultMessage: 'Package',
	},
	catalogValue: {
		id: 'create-project-version.catalog-value',
		defaultMessage: 'Publication',
	},
	usePackageValue: {
		id: 'create-project-version.use-package-value',
		defaultMessage: 'Use package value',
	},
	updatePackageHint: {
		id: 'create-project-version.update-package-hint',
		defaultMessage: 'To keep the publication value, update the JSON in the ZIP and select the package again. The archive is not modified automatically.',
	},
	versionType: {
		id: 'create-project-version.create-modal.stage.details.version-type',
		defaultMessage: 'Version type',
	},
	versionNumber: {
		id: 'create-project-version.create-modal.stage.details.version-number',
		defaultMessage: 'Version number',
	},
	versionNumberPlaceholder: {
		id: 'create-project-version.create-modal.stage.details.version-number-placeholder',
		defaultMessage: 'Enter version number, e.g. 1.2.3-alpha.1',
	},
	versionNumberDescription: {
		id: 'create-project-version.create-modal.stage.details.version-number-description',
		defaultMessage: 'The version number differentiates this specific version from others.',
	},
	versionSubtitle: {
		id: 'create-project-version.create-modal.stage.details.version-subtitle',
		defaultMessage: 'Version subtitle',
	},
	versionSubtitlePlaceholder: {
		id: 'create-project-version.create-modal.stage.details.version-subtitle-placeholder',
		defaultMessage: 'Enter subtitle...',
	},
	versionChangelog: {
		id: 'create-project-version.create-modal.stage.details.version-changelog',
		defaultMessage: 'Version changelog',
	},
	schematicMetadata: {
		id: 'create-project-version.schematic.metadata',
		defaultMessage: 'Schematic information',
	},
	schematicMetadataDescription: {
		id: 'create-project-version.sfbp.metadata-description',
		defaultMessage:
			'The .sfbp file is uploaded unchanged. These details describe the ShroudForge World Editor blueprint on ShroudEdit.',
	},
	schematicFormatVersion: {
		id: 'create-project-version.schematic.format-version',
		defaultMessage: 'Format version',
	},
	worldEditorVersion: {
		id: 'create-project-version.schematic.world-editor-version',
		defaultMessage: 'World Editor version',
	},
	worldEditorVersionPlaceholder: {
		id: 'create-project-version.schematic.world-editor-version-placeholder',
		defaultMessage: 'For example 0.4.0',
	},
	schematicWidth: {
		id: 'create-project-version.schematic.width',
		defaultMessage: 'Width',
	},
	schematicHeight: {
		id: 'create-project-version.schematic.height',
		defaultMessage: 'Height',
	},
	schematicDepth: {
		id: 'create-project-version.schematic.depth',
		defaultMessage: 'Depth',
	},
	installation: {
		id: 'create-project-version.schematic.installation',
		defaultMessage: 'Installation instructions',
	},
	installationPlaceholder: {
		id: 'create-project-version.schematic.installation-placeholder',
		defaultMessage: 'Explain how to import this file with the World Editor...',
	},
	previewDescription: {
		id: 'create-project-version.schematic.preview-description',
		defaultMessage: 'The featured image from the project gallery is used as the schematic preview.',
	},
	metadataTab: {
		id: 'create-project-version.create-modal.stage.details.metadata-tab',
		defaultMessage: 'Metadata',
	},
	detailsTab: {
		id: 'create-project-version.create-modal.stage.details.details-tab',
		defaultMessage: 'Details',
	},
	filesTab: {
		id: 'create-project-version.create-modal.stage.details.files-tab',
		defaultMessage: 'Files',
	},
})

type ComparableField = 'name' | 'version_number' | 'version_type' | 'game_versions' | 'environment' | 'changelog' | 'loaders'
const comparableFields: { key: ComparableField; label: string }[] = [
	{ key: 'name', label: 'Version subtitle' },
	{ key: 'version_number', label: 'Version number' },
	{ key: 'version_type', label: 'Release channel' },
	{ key: 'game_versions', label: 'Game versions' },
	{ key: 'environment', label: 'Environment' },
	{ key: 'changelog', label: 'Changelog' },
	{ key: 'loaders', label: 'Loaders' },
]

function displayValue(value: unknown): string {
	if (Array.isArray(value)) return value.join(', ')
	return value == null || value === '' ? '—' : String(value)
}

const packageIdMismatch = computed(() => Boolean(
	inferredVersionData.value?.mod_id &&
	projectV2.value.slug &&
	inferredVersionData.value.mod_id !== projectV2.value.slug.toLowerCase(),
))

const packageDifferences = computed(() => comparableFields.flatMap(({ key, label }) => {
	if (!inferredVersionData.value?.package_fields?.includes(key)) return []
	const packageValue = inferredVersionData.value[key]
	const catalogValue = draftVersion.value[key]
	if (JSON.stringify(packageValue) === JSON.stringify(catalogValue)) return []
	return [{ key, label, packageValue: displayValue(packageValue), catalogValue: displayValue(catalogValue) }]
}))

function usePackageValue(key: ComparableField) {
	const value = inferredVersionData.value?.[key]
	if (value === undefined) return
	;(draftVersion.value as unknown as Record<string, unknown>)[key] = structuredClone(value)
}

const dimensions = [
	{ key: 'schematic_width' as const, label: messages.schematicWidth },
	{ key: 'schematic_height' as const, label: messages.schematicHeight },
	{ key: 'schematic_depth' as const, label: messages.schematicDepth },
]

watch(
	projectType,
	(type) => {
		if (type === 'schematic') draftVersion.value.schematic_format_version ??= 8
	},
	{ immediate: true },
)

const editTabs = computed<TabsTab[]>(() => [
	{ label: formatMessage(messages.metadataTab), value: 'metadata' },
	{ label: formatMessage(messages.detailsTab), value: 'add-details' },
	{ label: formatMessage(messages.filesTab), value: 'add-files' },
])

function setEditTab(tab: TabsTab) {
	modal.value?.setStage(tab.value)
}

async function onImageUpload(file: File) {
	const response = await useImageUpload(file, { context: 'version' })
	return response.url
}
</script>
