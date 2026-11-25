<template>
	<ModerationProjectCard :project="queueEntry.project" :ownership="queueEntry.ownership">
		<template #badges>
			<div
				v-if="queueEntry.project.requested_status"
				class="flex items-center gap-2 rounded-full border border-solid border-surface-5 bg-surface-4 px-2.5 py-1"
			>
				<span class="text-sm text-secondary">Requesting</span>
				<Badge :type="queueEntry.project.requested_status" class="text-sm" />
			</div>
			<div
				v-if="showExternalDependencies"
				v-tooltip="'External dependencies'"
				class="flex items-center gap-1 rounded-full border border-solid border-surface-5 bg-surface-4 px-2.5 py-1"
			>
				<FileIcon aria-hidden="true" class="size-4 text-secondary" />
				<span class="text-sm font-medium text-secondary">
					{{ queueEntry.external_dependencies_count }}
				</span>
			</div>
		</template>
		<template #summary>
			<span
				v-tooltip="`Since ${formatDateTimeFull(queuedDate.toDate())}`"
				class="text-base text-secondary"
				:class="{
					'text-red': daysInQueue > 4,
					'text-orange': daysInQueue > 2 && daysInQueue <= 4,
				}"
			>
				{{ formattedDate }}
			</span>
		</template>
		<template #actions>
			<IconButton
				v-tooltip="'Begin review'"
				type="colored"
				color="orange"
				:label="'Begin review'"
				@click="openProjectForReview"
			>
				<ScaleIcon />
			</IconButton>
		</template>
	</ModerationProjectCard>
</template>

<script setup lang="ts">
import { FileIcon, ScaleIcon } from '@shroudedit/assets'
import { Badge, IconButton, useFormatDateTime, useRelativeTime } from '@shroudedit/ui'
import dayjs from 'dayjs'

import ModerationProjectCard from '~/components/ui/moderation/ModerationProjectCard.vue'
import type { ModerationProject } from '~/helpers/moderation'

const formatRelativeTime = useRelativeTime()
const formatDateTimeFull = useFormatDateTime({
	weekday: 'short',
	year: 'numeric',
	month: 'short',
	day: 'numeric',
	hour: 'numeric',
	minute: '2-digit',
	second: '2-digit',
	timeZoneName: 'short',
	timeZone: 'UTC',
})

const props = defineProps<{
	queueEntry: ModerationProject
	showExternalDependencies?: boolean
}>()

const emit = defineEmits<{
	startFromProject: [projectId: string]
}>()

const queuedDate = computed(() =>
	dayjs(
		props.queueEntry.project.queued ||
			props.queueEntry.project.published ||
			props.queueEntry.project.updated,
	),
)
const daysInQueue = computed(() =>
	Math.floor((Date.now() - queuedDate.value.toDate().getTime()) / (1000 * 60 * 60 * 24)),
)
const formattedDate = computed(() => {
	const date =
		props.queueEntry.project.queued ||
		props.queueEntry.project.published ||
		props.queueEntry.project.updated
	if (!date) return 'Unknown'

	try {
		return formatRelativeTime(dayjs(date).toISOString())
	} catch {
		return 'Unknown'
	}
})

function openProjectForReview() {
	emit('startFromProject', props.queueEntry.project.id)
}
</script>
