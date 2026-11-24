<template>
	<div class="flex flex-col gap-2.5">
		<span class="font-semibold text-contrast">Compatible loaders</span>

		<div class="flex flex-wrap gap-2">
			<TagItem
				v-for="loader in sortedLoaders"
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

		<span>Select one or more loaders this version supports.</span>
	</div>
</template>

<script lang="ts" setup>
import type { Labrinth } from '@shroudedit/api-client'
import { getLoaderIcon } from '@shroudedit/assets'
import { FormattedTag, TagItem } from '@shroudedit/ui'

const selectedLoaders = defineModel<string[]>({ default: [] })

const { loaders } = defineProps<{
	loaders: Labrinth.Tags.v2.Loader[]
	toggleLoader: (loader: string) => void
}>()

const loaderOrder = ['shroudtopia', 'shroudforge', 'eml']

const sortedLoaders = computed(() =>
	[...loaders].sort((a, b) => {
		const aIndex = loaderOrder.indexOf(a.name.toLowerCase())
		const bIndex = loaderOrder.indexOf(b.name.toLowerCase())
		return (
			(aIndex === -1 ? loaderOrder.length : aIndex) -
			(bIndex === -1 ? loaderOrder.length : bIndex)
		)
	}),
)
</script>
