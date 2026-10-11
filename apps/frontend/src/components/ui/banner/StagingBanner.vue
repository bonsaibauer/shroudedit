<script setup lang="ts">
import { XIcon } from '@shroudedit/assets'
import { commonMessages, defineMessages, IconButton, PagewideBanner, useVIntl } from '@shroudedit/ui'

const { formatMessage } = useVIntl()
const cosmetics = useCosmetics()
const flags = useFeatureFlags()

const messages = defineMessages({
	title: {
		id: 'layout.banner.staging.title',
		defaultMessage: 'You’re viewing ShroudEdit’s staging environment',
	},
	description: {
		id: 'layout.banner.staging.description',
		defaultMessage:
			'The staging environment is completely separate from the production ShroudEdit database. It is used for testing and may run newer backend or frontend versions than production.',
	},
})

function hideStagingBanner() {
	cosmetics.value.hideStagingBanner = true
}
</script>

<template>
	<PagewideBanner v-if="flags.showAllBanners || !cosmetics.hideStagingBanner" variant="warning">
		<template #title>
			<span>{{ formatMessage(messages.title) }}</span>
		</template>
		<template #description>
			{{ formatMessage(messages.description) }}
		</template>
		<template #actions_top_right>
			<IconButton
				type="quiet"
				:label="formatMessage(commonMessages.closeButton)"
				@click="hideStagingBanner"
			>
				<XIcon aria-hidden="true" />
			</IconButton>
		</template>
	</PagewideBanner>
</template>
