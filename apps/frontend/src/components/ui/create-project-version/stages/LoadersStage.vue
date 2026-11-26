<template>
	<div class="space-y-6">
		<LoaderPicker
			v-model="draftVersion.loaders"
			:loaders="availableLoaders"
			:toggle-loader="toggleLoader"
		/>

		<div v-if="draftVersion.loaders.length" class="space-y-1">
			<div class="flex items-center justify-between">
				<span class="font-semibold text-contrast"> Added loaders </span>
				<Button type="quiet" @click="onClearAll()">Clear all</Button>
			</div>
			<div
				class="flex flex-col gap-1.5 gap-y-4 rounded-xl border border-solid border-surface-5 p-3 py-4"
			>
				<div class="flex flex-wrap gap-2">
					<template
						v-for="loader in draftVersion.loaders.map((loaderName) =>
							loaders.find((loader) => loaderName === loader.name),
						)"
					>
						<TagItem
							v-if="loader"
							:key="`loader-${loader.name}`"
							:action="() => toggleLoader(loader.name)"
							class="border !border-solid border-surface-5 !transition-all hover:bg-button-bgHover hover:no-underline"
							:style="`--_color: var(--color-platform-${loader.name})`"
						>
							<component :is="getLoaderIcon(loader.name)" v-if="getLoaderIcon(loader.name)" />
							<FormattedTag :tag="loader.name" enforce-type="loader" />
							<XIcon class="text-secondary" />
						</TagItem>
					</template>
				</div>
			</div>
		</div>
		<Admonition v-if="draftVersion.loaders.includes('eml')" type="warning">
			{{ formatMessage(messages.emlCompatibilityWarning) }}
		</Admonition>
	</div>
</template>

<script lang="ts" setup>
import { getLoaderIcon, XIcon } from '@shroudedit/assets'
import { Admonition, Button, defineMessages, FormattedTag, TagItem, useVIntl } from '@shroudedit/ui'

import { injectManageVersionContext } from '~/providers/version/manage-version-modal'

import LoaderPicker from '../components/LoaderPicker.vue'

const generatedState = useGeneratedState()

const { formatMessage } = useVIntl()

const messages = defineMessages({
	emlCompatibilityWarning: {
		id: 'create-project-version.create-modal.stage.loaders.eml-compatibility-warning',
		defaultMessage:
			'EML mods also work with Shroudforge. Select Shroudforge too if you want this version listed for Shroudforge users.',
	},
})

const loaders = computed(() => generatedState.value.loaders)

const { draftVersion, projectType } = injectManageVersionContext()

const availableLoaders = computed(() =>
	projectType.value === 'modpack'
		? generatedState.value.loaders.filter((loader) => loader.name === 'sfpack')
		: projectType.value === 'schematic'
		? generatedState.value.loaders.filter((loader) =>
				['shroudtopia', 'shroudforge', 'eml'].includes(loader.name),
			)
		: generatedState.value.loaders,
)

const toggleLoader = (loader: string) => {
	if (draftVersion.value.loaders.includes(loader)) {
		draftVersion.value.loaders = draftVersion.value.loaders.filter((l) => l !== loader)
	} else {
		draftVersion.value.loaders = [...draftVersion.value.loaders, loader]
	}
}

const onClearAll = () => {
	draftVersion.value.loaders = []
}
</script>
