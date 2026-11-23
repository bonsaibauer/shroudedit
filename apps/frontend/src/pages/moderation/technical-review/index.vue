<script setup lang="ts">
import type { Labrinth } from '@shroudedit/api-client'
import { ButtonLink, defineMessages, injectShroudEditClient, useVIntl } from '@shroudedit/ui'
import { useQuery } from '@tanstack/vue-query'

const { formatMessage } = useVIntl()
const messages = defineMessages({
	pageTitle: {
		id: 'moderation.virusTotal.queue.title',
		defaultMessage: 'VirusTotal review queue - ShroudEdit',
	},
	heading: {
		id: 'moderation.virusTotal.queue.heading',
		defaultMessage: 'VirusTotal technical review',
	},
	projectCount: {
		id: 'moderation.virusTotal.queue.projectCount',
		defaultMessage:
			'{count, plural, one {# project awaiting a decision} other {# projects awaiting a decision}}',
	},
	searchPlaceholder: {
		id: 'moderation.virusTotal.queue.searchPlaceholder',
		defaultMessage: 'Search projects and files',
	},
	searchLabel: {
		id: 'moderation.virusTotal.queue.searchLabel',
		defaultMessage: 'Search technical reviews',
	},
	refresh: { id: 'moderation.virusTotal.queue.refresh', defaultMessage: 'Refresh' },
	loading: {
		id: 'moderation.virusTotal.queue.loading',
		defaultMessage: 'Loading VirusTotal review queue…',
	},
	loadError: {
		id: 'moderation.virusTotal.queue.loadError',
		defaultMessage: 'Could not load the review queue.',
	},
	empty: {
		id: 'moderation.virusTotal.queue.empty',
		defaultMessage: 'No projects are waiting for VirusTotal review.',
	},
	unknownOwner: { id: 'moderation.virusTotal.queue.unknownOwner', defaultMessage: 'Unknown owner' },
	fileCount: {
		id: 'moderation.virusTotal.queue.fileCount',
		defaultMessage: '{count, plural, one {# file} other {# files}}',
	},
	detectionCount: {
		id: 'moderation.virusTotal.queue.detectionCount',
		defaultMessage:
			'{count, plural, one {# malicious or suspicious detection} other {# malicious or suspicious detections}}',
	},
	versionCount: {
		id: 'moderation.virusTotal.queue.versionCount',
		defaultMessage: '{count, plural, one {# version} other {# versions}}',
	},
})

useHead({ title: formatMessage(messages.pageTitle) })

const client = injectShroudEditClient()
const search = ref('')

const { data, isLoading, error, refetch } = useQuery({
	queryKey: ['virustotal-review-queue'],
	queryFn: () =>
		client.labrinth.tech_review_internal.searchProjects({
			limit: 100,
			page: 0,
			sort_by: 'detection_count_desc',
			filter: {},
		}),
	refetchInterval: 60_000,
})

const projects = computed(() => {
	const response = data.value
	if (!response) return []
	const query = search.value.trim().toLowerCase()
	return response.project_reports
		.map((report) => ({
			report,
			project: response.projects[report.project_id],
			owner: response.ownership[report.project_id],
			thread: response.threads[response.projects[report.project_id]?.thread_id],
		}))
		.filter((item) => item.project && item.thread)
		.filter((item) => {
			if (!query) return true
			return [
				item.project.name,
				item.owner?.name,
				...item.report.versions.flatMap((version) => version.files.map((file) => file.file_name)),
			].some((value) => value?.toLowerCase().includes(query))
		})
})

function getFileCount(report: Labrinth.TechReview.Internal.ProjectReport): number {
	return report.versions.reduce((count, version) => count + version.files.length, 0)
}
</script>

<template>
	<div class="flex flex-col gap-4">
		<div class="flex flex-wrap items-center justify-between gap-3">
			<div>
				<h2 class="m-0 text-2xl font-bold">{{ formatMessage(messages.heading) }}</h2>
				<p class="mb-0 mt-1 text-secondary">
					{{ formatMessage(messages.projectCount, { count: projects.length }) }}
				</p>
			</div>
			<div class="flex flex-wrap gap-2">
				<input
					v-model="search"
					class="input-base min-w-56"
					:placeholder="formatMessage(messages.searchPlaceholder)"
					:aria-label="formatMessage(messages.searchLabel)"
				/>
				<ButtonLink type="outlined" @click="refetch()">{{
					formatMessage(messages.refresh)
				}}</ButtonLink>
			</div>
		</div>

		<div v-if="isLoading" class="rounded-xl bg-surface-2 p-6 text-secondary">
			{{ formatMessage(messages.loading) }}
		</div>
		<div v-else-if="error" class="rounded-xl bg-surface-2 p-6 text-red">
			{{ formatMessage(messages.loadError) }}
		</div>
		<div v-else-if="projects.length === 0" class="rounded-xl bg-surface-2 p-6 text-secondary">
			{{ formatMessage(messages.empty) }}
		</div>

		<div v-else class="flex flex-col gap-3">
			<NuxtLink
				v-for="item in projects"
				:key="item.project.id"
				:to="`/moderation/technical-review/${item.project.id}`"
				class="flex flex-wrap items-center justify-between gap-4 rounded-xl bg-surface-2 p-4 text-contrast no-underline hover:bg-surface-3"
			>
				<div class="flex min-w-0 flex-col gap-1">
					<strong class="truncate text-lg">{{ item.project.name }}</strong>
					<span class="text-sm text-secondary">
						{{ item.owner?.name ?? formatMessage(messages.unknownOwner) }} ·
						{{ item.project.status }}
					</span>
					<span class="text-sm text-secondary">
						{{ formatMessage(messages.fileCount, { count: getFileCount(item.report) }) }}
						·
						{{
							formatMessage(messages.detectionCount, {
								count: item.report.max_detection_count ?? 0,
							})
						}}
					</span>
				</div>
				<span class="rounded-full bg-surface-4 px-3 py-1 text-sm text-secondary">
					{{ formatMessage(messages.versionCount, { count: item.report.versions.length }) }}
				</span>
			</NuxtLink>
		</div>
	</div>
</template>
