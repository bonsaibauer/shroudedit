<script setup lang="ts">
import type { Labrinth } from '@shroudedit/api-client'
import {
	BookmarkIcon,
	DownloadIcon,
	GridIcon,
	HeartIcon,
	ImageIcon,
	ListIcon,
	MoreVerticalIcon,
	QuestionMarkIcon,
} from '@shroudedit/assets'
import type { CardAction } from '@shroudedit/ui'
import {
	BrowsePageLayout,
	BrowseSidebar,
	defineMessages,
	formatProjectTypeSentence,
	injectShroudEditClient,
	injectUserPreferences,
	provideBrowseManager,
	useBrowseSearch,
	useDebugLogger,
	useVIntl,
} from '@shroudedit/ui'
import { commonMessages } from '@shroudedit/ui/src/utils/common-messages'
import { cycleValue } from '@shroudedit/utils'
import { useQueryClient } from '@tanstack/vue-query'
import { useTimeoutFn } from '@vueuse/core'
import { computed, ref, watch } from 'vue'

import LogoAnimated from '~/components/brand/LogoAnimated.vue'
import AdPlaceholder from '~/components/ui/AdPlaceholder.vue'
import { projectQueryOptions, warmProjectCheckCaches } from '~/composables/queries/project'
import { withLabrinthCanaryHeader } from '~/helpers/canary.ts'
import type { DisplayLocation, DisplayMode } from '~/plugins/cosmetics.ts'

const { formatMessage } = useVIntl()
const debug = useDebugLogger('Discover')

const { updateDiscoverFilterContext } = useCdnDownloadContext()

const client = injectShroudEditClient()
const { updatePreferences } = injectUserPreferences()
const queryClient = useQueryClient()

const filtersMenuOpen = ref(false)
const route = useRoute()

const cosmetics = useCosmetics()
const tags = useGeneratedState()
const flags = useFeatureFlags()
const auth = await useAuth()

let prefetchTimeout: ReturnType<typeof useTimeoutFn> | null = null
const HOVER_DURATION_TO_PREFETCH_MS = 500

const handleProjectMouseEnter = (result: Labrinth.Search.v3.ResultSearchProject) => {
	const projectId = result.project_id
	prefetchTimeout = useTimeoutFn(
		() => {
			warmProjectCheckCaches(queryClient, result)
			queryClient.prefetchQuery(projectQueryOptions.v2(projectId, client))
			queryClient.prefetchQuery(projectQueryOptions.v3(projectId, client))
			queryClient.prefetchQuery(projectQueryOptions.members(projectId, client))
			queryClient.prefetchQuery(projectQueryOptions.dependencies(projectId, client))
			queryClient.prefetchQuery(projectQueryOptions.versionsV3(projectId, client))
		},
		HOVER_DURATION_TO_PREFETCH_MS,
		{ immediate: false },
	)
	prefetchTimeout.start()
}

const handleServerProjectMouseEnter = (result: Labrinth.Search.v3.ResultSearchProject) => {
	handleProjectMouseEnter(result)
}

const handleProjectHoverEnd = () => {
	if (prefetchTimeout) prefetchTimeout.stop()
}

const currentType = computed(() =>
	queryAsStringOrEmpty(route.params.type).replaceAll(/^\/|s\/?$/g, ''),
)

debug('initial route.params.type:', route.params.type, '→ currentType:', currentType.value)

const isServerType = computed(() => currentType.value === 'server')

const projectType = computed(() => tags.value.projectTypes.find((x) => x.id === currentType.value))

watch(
	() => projectType.value?.id,
	(val) => debug('projectType.id changed:', val),
)

const resultsDisplayLocation = computed<DisplayLocation | undefined>(
	() => projectType.value?.id as DisplayLocation,
)
const resultsDisplayMode = computed<DisplayMode>(() =>
	resultsDisplayLocation.value
		? cosmetics.value.searchDisplayMode[resultsDisplayLocation.value]
		: 'list',
)

const layoutPreferenceKeys = {
	mod: 'mods',
	plugin: 'plugins',
	datapack: 'datapacks',
	shader: 'shaders',
	resourcepack: 'resourcepacks',
	modpack: 'modpacks',
	schematic: 'mods',
	server: 'servers',
	user: 'users',
} as const satisfies Partial<Record<DisplayLocation, keyof Labrinth.Users.v3.LayoutPreferences>>

const maxResultsForView = ref<Record<DisplayMode, number[]>>({
	list: [5, 10, 15, 20, 50, 100],
	grid: [6, 12, 18, 24, 48, 96],
	gallery: [6, 10, 16, 20, 50, 100],
})

const currentMaxResultsOptions = computed(
	() => maxResultsForView.value[resultsDisplayMode.value] ?? [20],
)

function cycleSearchDisplayMode() {
	if (!resultsDisplayLocation.value) return
	const displayMode = cycleValue(
		cosmetics.value.searchDisplayMode[resultsDisplayLocation.value],
		tags.value.projectViewModes.filter((x) => x !== 'grid'),
	)
	cosmetics.value.searchDisplayMode[resultsDisplayLocation.value] = displayMode

	const preferenceKey =
		layoutPreferenceKeys[resultsDisplayLocation.value as keyof typeof layoutPreferenceKeys]
	if (!preferenceKey) return

	void updatePreferences({
		layouts: {
			[preferenceKey]: displayMode === 'list' ? 'rows' : 'grid',
		} as Partial<Labrinth.Users.v3.LayoutPreferences>,
	}).catch(() => undefined)
}

function parseSearchParams(requestParams: string): Labrinth.Search.SearchParams {
	const params = new URLSearchParams(requestParams.replace(/^\?/, ''))

	return {
		query: params.get('query') ?? undefined,
		offset: params.get('offset') ?? undefined,
		index: params.get('index') ?? undefined,
		limit: params.get('limit') ?? undefined,
		new_filters: params.get('new_filters') ?? undefined,
	}
}

// Search returns expanded dependency data that no card renders and that isn't part of
// ResultSearchProject. On modpacks it is ~90% of the response, and everything cached here
// is serialized into the SSR payload, so drop it before it reaches the query cache.
function stripUnrenderedFields(
	hits: Labrinth.Search.v3.ResultSearchProject[],
): Labrinth.Search.v3.ResultSearchProject[] {
	return hits.map((hit) => {
		const { dependencies, dependency_project_ids, compatible_dependency_project_ids, ...rendered } =
			hit as Labrinth.Search.v3.ResultSearchProject & Record<string, unknown>
		return rendered as Labrinth.Search.v3.ResultSearchProject
	})
}

async function fetchSearch(requestParams: string) {
	debug('search() called', {
		requestParams: requestParams.substring(0, 100),
		isServer: isServerType.value,
		projectTypeId: projectTypeId.value,
	})

	const raw = await client.labrinth.projects_v3.search(parseSearchParams(requestParams), {
		headers: withLabrinthCanaryHeader(),
	})

	debug('search() response', { total_hits: raw.total_hits, hitCount: raw.hits?.length })

	const hits = stripUnrenderedFields(raw.hits ?? [])

	if (isServerType.value) {
		return {
			projectHits: [],
			serverHits: hits,
			total_hits: raw.total_hits,
			per_page: raw.hits_per_page,
		}
	}

	return {
		projectHits: hits,
		serverHits: [],
		total_hits: raw.total_hits,
		per_page: raw.hits_per_page,
	}
}

async function search(requestParams: string) {
	return await queryClient.ensureQueryData({
		queryKey: ['discover', 'search', 'v3', requestParams],
		queryFn: () => fetchSearch(requestParams),
		staleTime: 30_000,
	})
}

function getCardActions(
	result: Labrinth.Search.v3.ResultSearchProject,
	currentProjectType: string,
): CardAction[] {
	if (currentProjectType === 'server') return []

	if (flags.value.showDiscoverProjectButtons) {
		return [
			{
				key: 'download',
				label: 'Download',
				icon: DownloadIcon,
				color: 'brand',
				onClick: () => {},
			},
			{
				key: 'heart',
				label: '',
				icon: HeartIcon,
				circular: true,
				onClick: () => {},
			},
			{
				key: 'bookmark',
				label: '',
				icon: BookmarkIcon,
				circular: true,
				onClick: () => {},
			},
			{
				key: 'more',
				label: '',
				icon: MoreVerticalIcon,
				circular: true,
				type: 'transparent',
				onClick: () => {},
			},
		]
	}

	return []
}

const messages = defineMessages({
	gameVersionProvidedByServer: {
		id: 'search.filter.locked.server-game-version.title',
		defaultMessage: 'Game version is provided by the server',
	},
	modLoaderProvidedByServer: {
		id: 'search.filter.locked.server-loader.title',
		defaultMessage: 'Loader is provided by the server',
	},
	providedByServer: {
		id: 'search.filter.locked.server',
		defaultMessage: 'Provided by the server',
	},
	syncFilterButton: {
		id: 'search.filter.locked.server.sync',
		defaultMessage: 'Sync with server',
	},
	seoTitle: {
		id: 'discover.seo.title',
		defaultMessage: 'Search {projectType}',
	},
	seoTitleWithQuery: {
		id: 'discover.seo.title-with-query',
		defaultMessage: 'Search {projectType} | {query}',
	},
	seoDescription: {
		id: 'discover.seo.description',
		defaultMessage:
			'Search and browse Enshrouded {projectType} on ShroudEdit with fast, accurate filters.',
	},
	gameVersionShaderMessage: {
		id: 'search.filter.game-version-shader-message',
		defaultMessage:
			'Shader packs for older versions most likely work on newer versions with only minor issues.',
	},
})

const advancedFiltersCollapsed = computed({
	get: () => flags.value.advancedFiltersCollapsed,
	set: (value) => {
		flags.value.advancedFiltersCollapsed = value
		saveFeatureFlags()
	},
})

const dismissedPhotosensitivityFilterWarning = computed({
	get: () => flags.value.dismissedPhotosensitivityFilterWarning,
	set: (value) => {
		flags.value.dismissedPhotosensitivityFilterWarning = value
		saveFeatureFlags()
	},
})

const projectTypeId = computed(() => projectType.value?.id ?? 'mod')

debug('projectTypeId:', projectTypeId.value)
watch(projectTypeId, (val) => debug('projectTypeId changed:', val))

const searchState = useBrowseSearch({
	projectType: projectTypeId,
	tags,
	search,
	maxResultsOptions: currentMaxResultsOptions,
	displayMode: resultsDisplayMode,
})

// Warm check caches for every visible hit so clicking a result skips /project/{slug}/check
watch(
	[() => searchState.projectHits.value, () => searchState.serverHits.value],
	([projectHits, serverHits]) => {
		warmProjectCheckCaches(queryClient, [...projectHits, ...serverHits])
	},
	{ immediate: true },
)

watch(
	() =>
		searchState.isServerType.value
			? searchState.serverCurrentFilters.value
			: searchState.currentFilters.value,
	(filters) => updateDiscoverFilterContext(filters),
	{ deep: true, immediate: true },
)

debug('calling initial refreshSearch')
await searchState.refreshSearch()

const ogTitle = computed(() =>
	searchState.query.value
		? formatMessage(messages.seoTitleWithQuery, {
				projectType: formatProjectTypeSentence(
					formatMessage,
					projectType.value?.id ?? 'project',
					2,
				),
				query: searchState.query.value,
			})
		: formatMessage(messages.seoTitle, {
				projectType: formatProjectTypeSentence(
					formatMessage,
					projectType.value?.id ?? 'project',
					2,
				),
			}),
)
const description = computed(() =>
	formatMessage(messages.seoDescription, {
		projectType: formatProjectTypeSentence(formatMessage, projectType.value?.id ?? 'project', 2),
	}),
)

useSeoMeta({
	description,
	ogTitle,
	ogDescription: description,
})

debug('calling provideBrowseManager')
provideBrowseManager({
	tags,
	projectType: projectTypeId,
	...searchState,
	getProjectLink: (result: Labrinth.Search.v3.ResultSearchProject) =>
		`/${projectType.value?.id ?? 'project'}/${result.slug ? result.slug : result.project_id}`,
	getServerProjectLink: (result: Labrinth.Search.v3.ResultSearchProject) =>
		`/server/${result.slug ?? result.project_id}`,
	selectableProjectTypes: computed(() => []),
	showProjectTypeTabs: computed(() => false),
	variant: 'web',
	getCardActions,
	advancedFiltersCollapsed,
	dismissedPhotosensitivityFilterWarning,
	displayMode: resultsDisplayMode,
	cycleDisplayMode: cycleSearchDisplayMode,
	maxResultsOptions: currentMaxResultsOptions,
	onProjectHover: handleProjectMouseEnter,
	onServerProjectHover: handleServerProjectMouseEnter,
	onProjectHoverEnd: handleProjectHoverEnd,
	filtersMenuOpen,
	lockedFilterMessages: {
		gameVersion: formatMessage(messages.gameVersionProvidedByServer),
		modLoader: formatMessage(messages.modLoaderProvidedByServer),
		syncButton: formatMessage(messages.syncFilterButton),
		providedBy: formatMessage(messages.providedByServer),
		gameVersionShaderMessage: formatMessage(messages.gameVersionShaderMessage),
	},
	loadingComponent: LogoAnimated,
})

</script>
<template>
	<Teleport v-if="flags.searchBackground" to="#absolute-background-teleport">
		<div class="search-background">
			<QuestionMarkIcon aria-hidden="true" class="search-background-question" />
		</div>
	</Teleport>

	<div
		class="grid min-w-0 gap-3"
		:class="
			cosmetics.rightSearchLayout
				? 'lg:grid-cols-[minmax(0,1fr)_18.75rem]'
				: 'lg:grid-cols-[18.75rem_minmax(0,1fr)]'
		"
	>
		<section
			class="flex min-w-0 flex-col gap-2"
			:class="['mt-6 sm:mt-0', cosmetics.rightSearchLayout ? 'lg:order-1' : 'lg:order-2']"
		>
			<BrowsePageLayout>
				<template #display-mode-icon>
					<GridIcon v-if="resultsDisplayMode === 'grid'" />
					<ImageIcon v-else-if="resultsDisplayMode === 'gallery'" />
					<ListIcon v-else />
				</template>
			</BrowsePageLayout>
		</section>

		<aside
			class="min-w-0"
			:class="cosmetics.rightSearchLayout ? 'lg:order-2' : 'lg:order-1'"
			:aria-label="formatMessage(commonMessages.filtersLabel)"
		>
			<BrowseSidebar>
				<template #prepend>
					<AdPlaceholder v-if="!auth.user" />
				</template>
			</BrowseSidebar>
		</aside>
	</div>

</template>
<style lang="scss" scoped>
.search-background {
	width: 100%;
	height: 20rem;
	display: flex;
	justify-content: flex-end;
	padding-right: 8vw;
	color: var(--color-brand);
	pointer-events: none;
	mask-image: linear-gradient(to bottom, black, transparent);
	opacity: 0.16;
}

.search-background-question {
	height: 100%;
	width: auto;
}
</style>
