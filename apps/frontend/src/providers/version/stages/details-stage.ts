import { LeftArrowIcon, PlusIcon, SaveIcon, SpinnerIcon, XIcon } from '@shroudedit/assets'
import type { StageConfigInput } from '@shroudedit/ui'
import { markRaw } from 'vue'

import DetailsStage from '~/components/ui/create-project-version/stages/DetailsStage.vue'

import type { ManageVersionContextValue } from '../manage-version-modal'

function hasValidSchematicMetadata(ctx: ManageVersionContextValue) {
	if (ctx.projectType.value !== 'schematic') return true

	const version = ctx.draftVersion.value
	const formatVersion = Number(version.schematic_format_version)
	return Boolean(
		Number.isInteger(formatVersion) &&
			formatVersion >= 1 &&
			formatVersion <= 2147483647 &&
			version.world_editor_version?.trim() &&
			version.schematic_width &&
			version.schematic_height &&
			version.schematic_depth,
	)
}

export const stageConfig: StageConfigInput<ManageVersionContextValue> = {
	id: 'add-details',
	stageContent: markRaw(DetailsStage),
	title: (ctx) => (ctx.editingVersion.value ? 'Edit version' : 'Details'),
	maxWidth: '744px',
	disableClose: (ctx) => ctx.isUploading.value,
	leftButtonConfig: (ctx) =>
		ctx.editingVersion.value
			? {
					label: 'Cancel',
					icon: XIcon,
					onClick: () => ctx.modal.value?.hide(),
				}
			: {
					label: 'Back',
					icon: LeftArrowIcon,
					disabled: ctx.isUploading.value,
					onClick: () => ctx.modal.value?.prevStage(),
				},
	rightButtonConfig: (ctx) => ({
		label: ctx.editingVersion.value
			? 'Save changes'
			: ctx.isUploading.value
				? ctx.uploadProgress.value.progress >= 1
					? 'Creating version'
					: `Uploading ${Math.round(ctx.uploadProgress.value.progress * 100)}%`
				: 'Create version',
		icon: ctx.isSubmitting.value ? SpinnerIcon : ctx.editingVersion.value ? SaveIcon : PlusIcon,
		iconPosition: 'before',
		iconClass: ctx.isSubmitting.value ? 'animate-spin' : undefined,
		color: 'green',
		disabled: ctx.isSubmitting.value || !hasValidSchematicMetadata(ctx),
		onClick: () =>
			ctx.editingVersion.value ? ctx.handleSaveVersionEdits() : ctx.handleCreateVersion(),
	}),
	nonProgressStage: (ctx) => ctx.editingVersion.value,
}
