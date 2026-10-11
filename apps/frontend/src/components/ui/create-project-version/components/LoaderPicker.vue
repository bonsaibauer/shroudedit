<template>
	<div class="flex flex-col gap-2.5">
		<span class="font-semibold text-contrast">Loaders</span>

		<Chips
			v-model="loaderGroup"
			:items="groupLabels"
			:never-empty="true"
			:capitalize="true"
			size="small"
		/>

		<div
			class="flex min-h-[150px] flex-1 flex-col gap-4 overflow-y-auto rounded-xl border border-solid border-surface-5 p-3"
		>
			<div v-if="groupedLoaders[loaderGroup].length" class="flex flex-col gap-1.5">
				<div class="flex flex-wrap gap-2">
					<TagItem
						v-for="loader in groupedLoaders[loaderGroup]"
						:key="`loader-${loader.name}`"
						:action="() => toggleLoader(loader.name)"
						class="border !border-solid !transition-all hover:bg-button-bgHover hover:no-underline"
						:class="
							selectedLoaders.includes(loader.name)
								? 'border-brand bg-brand-highlight text-brand'
								: 'border-surface-5'
						"
						:style="`--_color: var(--color-platform-${loader.name})`"
					>
						<component :is="getLoaderIcon(loader.name)" v-if="getLoaderIcon(loader.name)" />
						<FormattedTag :tag="loader.name" enforce-type="loader" />
					</TagItem>
				</div>
			</div>
		</div>

		<span>Select one or more loaders this version supports.</span>
	</div>
</template>

<script lang="ts" setup>
import type { Labrinth } from '@shroudedit/api-client'
import { getLoaderIcon } from '@shroudedit/assets'
import { Chips, FormattedTag, TagItem } from '@shroudedit/ui'

const selectedLoaders = defineModel<string[]>({ default: [] })

const { loaders } = defineProps<{
	loaders: Labrinth.Tags.v2.Loader[]
	toggleLoader: (loader: string) => void
}>()

const loaderGroup = ref<GroupLabels>('mods')

type GroupLabels = 'mods' | 'plugins' | 'packs' | 'shaders' | 'other'

const groupLabels: GroupLabels[] = ['mods', 'plugins', 'packs', 'shaders']

function groupLoaders(loaders: Labrinth.Tags.v2.Loader[]) {
	const groups: Record<GroupLabels, Labrinth.Tags.v2.Loader[]> = {
		mods: [],
		plugins: [],
		packs: [],
		shaders: [],
		other: [],
	}

	const MOD_SORT = ['shroudtopia', 'shroudforge', 'eml']

	for (const loader of loaders) {
		const name = loader.name.toLowerCase()
		if (MOD_SORT.includes(name)) groups.mods.push(loader)
		else groups.other.push(loader)
	}

	groups.mods.sort((a, b) => MOD_SORT.indexOf(a.name) - MOD_SORT.indexOf(b.name))

	return groups
}

const groupedLoaders = computed(() => groupLoaders(loaders))

onMounted(() => {
	if (selectedLoaders.value.length === 0) return

	const groups = groupedLoaders.value
	for (const [groupName, loadersInGroup] of Object.entries(groups)) {
		if (loadersInGroup.some((loader) => selectedLoaders.value.includes(loader.name))) {
			loaderGroup.value = groupName as GroupLabels
			break
		}
	}
})
</script>
