<template>
	<div class="flex flex-col gap-4">
		<ModerationQueueToolbar
			v-model="query"
			:page="currentPage"
			:total-pages="totalPages"
			:loading="pending"
			@search="goToPage(1)"
			@switch-page="goToPage"
		>
			<template #actions>
				<Combobox
					v-model="currentScope"
					class="!w-full flex-grow sm:!w-[220px] sm:flex-grow-0"
					trigger-type="base"
					trigger-size="lg"
					:options="scopeOptions"
					:placeholder="formatMessage(commonMessages.filterByLabel)"
					@select="goToPage(1)"
				>
					<template #selected>
						<span class="flex flex-row gap-2 align-middle font-semibold">
							<ListFilterIcon class="size-5 flex-shrink-0 text-secondary" />
							<ModerationFilterCount
								:label="currentScopeLabel"
								:count="totalCount"
								:loading="pending"
							/>
						</span>
					</template>
				</Combobox>

				<Combobox
					v-model="currentProjectType"
					class="!w-full flex-grow sm:!w-[190px] sm:flex-grow-0"
					trigger-type="base"
					trigger-size="lg"
					:options="projectTypeOptions"
					:placeholder="formatMessage(commonMessages.filterByLabel)"
					@select="goToPage(1)"
				>
					<template #selected="{ label }">
						<span class="flex flex-row gap-2 align-middle font-semibold">
							<ListFilterIcon class="size-5 flex-shrink-0 text-secondary" />
							<span class="truncate text-contrast">{{ label }}</span>
						</span>
					</template>
				</Combobox>

				<Combobox
					v-model="currentSort"
					class="!w-full flex-grow sm:!w-[200px] sm:flex-grow-0"
					trigger-type="base"
					trigger-size="lg"
					:options="sortOptions"
					:placeholder="formatMessage(commonMessages.sortByLabel)"
					@select="goToPage(1)"
				>
					<template #selected="{ label }">
						<span class="flex flex-row gap-2 align-middle font-semibold">
							<SortAscIcon
								v-if="currentSort === 'created_asc' || currentSort === 'detection_count_asc'"
								class="size-5 flex-shrink-0 text-secondary"
							/>
							<SortDescIcon v-else class="size-5 flex-shrink-0 text-secondary" />
							<span class="truncate text-contrast">{{ label }}</span>
						</span>
					</template>
				</Combobox>

				<Combobox
					v-model="itemsPerPage"
					class="!w-full flex-grow sm:!w-[140px] sm:flex-grow-0"
					trigger-type="base"
					trigger-size="lg"
					:options="itemsPerPageOptions"
					:placeholder="formatMessage(messages.itemsPerPage)"
					@select="goToPage(1)"
				>
					<template #selected>
						<span class="font-semibold text-contrast">{{
							formatMessage(messages.items, { count: itemsPerPage })
						}}</span>
					</template>
				</Combobox>
			</template>
			<template #meta>
				<div v-if="totalCount > 0">
					{{
						formatMessage(messages.showing, {
							start: formatNumber(pageStart),
							end: formatNumber(pageEnd),
							total: formatNumber(totalCount),
						})
					}}
				</div>
			</template>
		</ModerationQueueToolbar>

		<ModerationQueueSkeleton v-if="pending" />
		<EmptyState
			v-else-if="error"
			type="no-tasks"
			:heading="formatMessage(messages.loadError)"
			:description="error.message"
		/>
		<EmptyState
			v-else-if="projects.length === 0"
			:type="query ? 'no-search-result' : 'no-tasks'"
			:heading="formatMessage(messages.empty)"
			:description="
				formatMessage(
					currentScope === 'needs_review'
						? messages.emptyQueueDescription
						: messages.emptyAllDescription,
				)
			"
		/>
		<div v-else class="flex flex-col gap-3">
			<ModerationProjectCard
				v-for="item in projects"
				:key="item.project.id"
				:project="item.project"
				:ownership="item.ownership ?? null"
			>
				<template #badges>
					<span
						v-if="item.project.needs_review"
						class="bg-orange-highlight rounded-full border border-solid border-orange px-2.5 py-1 text-sm font-medium text-orange"
					>
						{{ formatMessage(messages.needsReview) }}
					</span>
				</template>
				<template #details>
					<span class="text-sm text-secondary">
						{{ formatMessage(messages.fileCount, { count: getFileCount(item.report) }) }}
						·
						{{
							formatMessage(messages.detectionCount, {
								count: item.report.max_detection_count ?? 0,
							})
						}}
					</span>
				</template>
				<template #actions>
					<ButtonLink
						type="colored"
						color="orange"
						:to="`/moderation/technical-review/${item.project.id}`"
					>
						{{ formatMessage(messages.openReview) }}
					</ButtonLink>
				</template>
			</ModerationProjectCard>
		</div>

		<div v-if="totalPages > 1" class="flex justify-end">
			<Pagination :page="currentPage" :count="totalPages" @switch-page="goToPage" />
		</div>
	</div>
</template>

<script setup lang="ts">
import type { Labrinth } from '@shroudedit/api-client'
import { ListFilterIcon, SortAscIcon, SortDescIcon } from '@shroudedit/assets'
import {
	ButtonLink,
	Combobox,
	type ComboboxOption,
	commonMessages,
	defineMessages,
	EmptyState,
	getProjectTypeCategoryMessage,
	injectShroudEditClient,
	Pagination,
	useFormatNumber,
	useVIntl,
} from '@shroudedit/ui'
import { useQuery } from '@tanstack/vue-query'

import ModerationFilterCount from '~/components/ui/moderation/ModerationFilterCount.vue'
import ModerationProjectCard from '~/components/ui/moderation/ModerationProjectCard.vue'
import ModerationQueueSkeleton from '~/components/ui/moderation/ModerationQueueSkeleton.vue'
import ModerationQueueToolbar from '~/components/ui/moderation/ModerationQueueToolbar.vue'
import { MODERATION_PROJECT_TYPES } from '~/helpers/moderation'

type Scope = 'needs_review' | 'all'
type Sort = Labrinth.TechReview.Internal.SearchProjectsSort
type ProjectType = 'all' | (typeof MODERATION_PROJECT_TYPES)[number]

const { formatMessage } = useVIntl()
const formatNumber = useFormatNumber()
const messages = defineMessages({
	pageTitle: {
		id: 'moderation.virusTotal.queue.title',
		defaultMessage: 'Technical review - ShroudEdit',
	},
	needsReview: { id: 'moderation.virusTotal.queue.needsReview', defaultMessage: 'Needs review' },
	allReports: { id: 'moderation.virusTotal.queue.allReports', defaultMessage: 'All tech reports' },
	allProjectTypes: {
		id: 'moderation.virusTotal.queue.allProjectTypes',
		defaultMessage: 'All project types',
	},
	oldest: { id: 'moderation.virusTotal.queue.oldest', defaultMessage: 'Oldest' },
	newest: { id: 'moderation.virusTotal.queue.newest', defaultMessage: 'Newest' },
	mostDetections: {
		id: 'moderation.virusTotal.queue.mostDetections',
		defaultMessage: 'Most detections',
	},
	leastDetections: {
		id: 'moderation.virusTotal.queue.leastDetections',
		defaultMessage: 'Least detections',
	},
	itemsPerPage: {
		id: 'moderation.virusTotal.queue.itemsPerPage',
		defaultMessage: 'Items per page',
	},
	items: { id: 'moderation.virusTotal.queue.items', defaultMessage: '{count} items' },
	showing: {
		id: 'moderation.virusTotal.queue.showing',
		defaultMessage: 'Showing {start}–{end} of {total} tech reports',
	},
	loadError: {
		id: 'moderation.virusTotal.queue.loadError',
		defaultMessage: 'Could not load technical reviews',
	},
	empty: { id: 'moderation.virusTotal.queue.empty', defaultMessage: 'No technical reviews found' },
	emptyQueueDescription: {
		id: 'moderation.virusTotal.queue.emptyQueueDescription',
		defaultMessage: 'No projects need technical review.',
	},
	emptyAllDescription: {
		id: 'moderation.virusTotal.queue.emptyAllDescription',
		defaultMessage: 'No technical reports match these filters.',
	},
	fileCount: {
		id: 'moderation.virusTotal.queue.fileCount',
		defaultMessage: '{count, plural, one {# file} other {# files}}',
	},
	detectionCount: {
		id: 'moderation.virusTotal.queue.detectionCount',
		defaultMessage:
			'{count, plural, one {# malicious or suspicious detection} other {# malicious or suspicious detections}}',
	},
	openReview: { id: 'moderation.virusTotal.queue.openReview', defaultMessage: 'Open tech review' },
})

useHead({ title: formatMessage(messages.pageTitle) })

const client = injectShroudEditClient()
const route = useRoute()
const router = useRouter()

const scopeOptions: ComboboxOption<Scope>[] = [
	{ value: 'needs_review', label: formatMessage(messages.needsReview) },
	{ value: 'all', label: formatMessage(messages.allReports) },
]
const projectTypeOptions: ComboboxOption<ProjectType>[] = [
	{ value: 'all', label: formatMessage(messages.allProjectTypes) },
	...MODERATION_PROJECT_TYPES.map((type) => ({
		value: type,
		label: formatMessage(getProjectTypeCategoryMessage(type)),
	})),
]
const sortOptions: ComboboxOption<Sort>[] = [
	{ value: 'detection_count_desc', label: formatMessage(messages.mostDetections) },
	{ value: 'created_asc', label: formatMessage(messages.oldest) },
	{ value: 'created_desc', label: formatMessage(messages.newest) },
	{ value: 'detection_count_asc', label: formatMessage(messages.leastDetections) },
]
const itemsPerPageOptions: ComboboxOption<number>[] = [20, 40, 60, 80, 100, 200].map((value) => ({
	value,
	label: String(value),
}))

function parseOption<T extends string>(
	value: unknown,
	options: ComboboxOption<T>[],
	fallback: T,
): T {
	const stringValue = typeof value === 'string' ? value : ''
	return options.some((option) => option.value === stringValue) ? (stringValue as T) : fallback
}

function parsePage(value: unknown): number {
	const page = Number(value)
	return Number.isInteger(page) && page > 0 ? page : 1
}

const query = ref(typeof route.query.q === 'string' ? route.query.q : '')
const currentScope = ref<Scope>(parseOption(route.query.scope, scopeOptions, 'needs_review'))
const currentProjectType = ref<ProjectType>(
	parseOption(route.query.projectType, projectTypeOptions, 'all'),
)
const currentSort = ref<Sort>(parseOption(route.query.sort, sortOptions, 'detection_count_desc'))
const currentPage = ref(parsePage(route.query.page))
const itemsPerPageCookie = useCookie<number>('moderation-items-per-page', {
	default: () => 40,
	maxAge: 60 * 60 * 24 * 365,
	sameSite: 'lax',
	path: '/',
})
const itemsPerPage = computed({
	get: () =>
		itemsPerPageOptions.some((option) => option.value === Number(itemsPerPageCookie.value))
			? Number(itemsPerPageCookie.value)
			: 40,
	set: (value: number) => {
		itemsPerPageCookie.value = value
	},
})
const currentScopeLabel = computed(
	() => scopeOptions.find((option) => option.value === currentScope.value)?.label ?? '',
)

watch([query, currentScope, currentProjectType, currentSort, currentPage], () => {
	const nextQuery = { ...route.query }
	if (query.value) nextQuery.q = query.value
	else delete nextQuery.q
	if (currentScope.value !== 'needs_review') nextQuery.scope = currentScope.value
	else delete nextQuery.scope
	if (currentProjectType.value !== 'all') nextQuery.projectType = currentProjectType.value
	else delete nextQuery.projectType
	if (currentSort.value !== 'detection_count_desc') nextQuery.sort = currentSort.value
	else delete nextQuery.sort
	if (currentPage.value > 1) nextQuery.page = String(currentPage.value)
	else delete nextQuery.page
	router.replace({ path: route.path, query: nextQuery })
})

watch(
	() => route.query,
	(nextQuery) => {
		query.value = typeof nextQuery.q === 'string' ? nextQuery.q : ''
		currentScope.value = parseOption(nextQuery.scope, scopeOptions, 'needs_review')
		currentProjectType.value = parseOption(nextQuery.projectType, projectTypeOptions, 'all')
		currentSort.value = parseOption(nextQuery.sort, sortOptions, 'detection_count_desc')
		currentPage.value = parsePage(nextQuery.page)
	},
)

const request = computed<Labrinth.TechReview.Internal.SearchProjectsRequest>(() => ({
	limit: itemsPerPage.value,
	page: currentPage.value - 1,
	scope: currentScope.value,
	query: query.value.trim() || undefined,
	sort_by: currentSort.value,
	filter: {
		project_type:
			currentProjectType.value === 'all'
				? []
				: [currentProjectType.value === 'server' ? 'enshrouded_server' : currentProjectType.value],
	},
}))
const { data, isPending, isPlaceholderData, error } = useQuery({
	queryKey: computed(() => ['virustotal-review-queue', request.value] as const),
	queryFn: ({ queryKey }) => client.labrinth.tech_review_internal.searchProjects(queryKey[1]),
	placeholderData: (previousData) => previousData,
	refetchInterval: 60_000,
})
const pending = computed(() => isPending.value || isPlaceholderData.value)
const totalCount = computed(() => data.value?.total_count ?? 0)
const totalPages = computed(() => Math.ceil(totalCount.value / itemsPerPage.value))
const projects = computed(() => {
	const response = data.value
	if (!response) return []
	return response.project_reports.flatMap((report) => {
		const project = response.projects[report.project_id]
		if (!project) return []
		return [{ report, project, ownership: response.ownership[report.project_id] }]
	})
})
const pageStart = computed(() =>
	totalCount.value === 0 ? 0 : (currentPage.value - 1) * itemsPerPage.value + 1,
)
const pageEnd = computed(() =>
	Math.min(totalCount.value, (currentPage.value - 1) * itemsPerPage.value + projects.value.length),
)

watch(totalPages, (pages) => {
	if (pending.value) return
	if (pages === 0 && currentPage.value !== 1) goToPage(1)
	else if (pages > 0 && currentPage.value > pages) goToPage(pages)
})

function goToPage(page: number) {
	currentPage.value = page
}

function getFileCount(report: Labrinth.TechReview.Internal.ProjectReport): number {
	return report.versions.reduce((count, version) => count + version.files.length, 0)
}
</script>
