<template>
	<div v-if="downloadRows.length > 0" class="flex flex-col gap-4">
		<div v-if="downloadRows.length > 0" class="flex flex-col gap-2.5">
			<div v-if="showTitle" class="flex flex-wrap items-center justify-between gap-2">
				<h3 class="m-0 flex items-center gap-1.5 text-base font-semibold text-contrast">
					{{ formatMessage(messages.dependenciesTitle) }}
					<InfoIcon
						v-if="duplicateDependencyRowsHidden"
						v-tooltip="formatMessage(messages.duplicateDependenciesHidden)"
						aria-hidden="true"
						class="size-4 text-secondary"
					/>
				</h3>
			</div>
			<div class="rounded-2xl bg-surface-2 p-2 pl-4 pr-3">
				<DownloadDependency
					v-for="dependency in downloadRows"
					:key="dependency.key"
					:dependency="dependency"
					@download="emit('download')"
				/>
			</div>
		</div>
	</div>
</template>

<script setup lang="ts">
import { InfoIcon } from '@shroudedit/assets'
import { defineMessages, useVIntl } from '@shroudedit/ui'

import { injectDownloadModalProvider } from './download-modal-provider'
import DownloadDependency from './DownloadDependency.vue'

defineOptions({
	name: 'DownloadDependencies',
})

withDefaults(
	defineProps<{
		showTitle?: boolean
	}>(),
	{
		showTitle: true,
	},
)

const emit = defineEmits<{
	download: []
}>()
const { formatMessage } = useVIntl()
const { duplicateDependencyRowsHidden, downloadRows } = injectDownloadModalProvider()

const messages = defineMessages({
	dependenciesTitle: {
		id: 'project.download.dependencies-title',
		defaultMessage: 'Dependencies',
	},
	duplicateDependenciesHidden: {
		id: 'project.download.duplicate-dependencies-hidden',
		defaultMessage: 'Duplicate dependencies are hidden',
	},
})
</script>
