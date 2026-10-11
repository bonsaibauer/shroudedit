<script setup lang="ts">
import { XIcon } from '@shroudedit/assets'
import {
	commonMessages,
	defineMessages,
	IconButton,
	PagewideBanner,
	useVIntl,
} from '@shroudedit/ui'

const { formatMessage } = useVIntl()
const cosmetics = useCosmetics()
const flags = useFeatureFlags()

const messages = defineMessages({
	title: {
		id: 'layout.banner.dev.title',
		defaultMessage: 'You’re viewing ShroudEdit’s Dev environment',
	},
	description: {
		id: 'layout.banner.dev.description',
		defaultMessage:
			'This Dev frontend uses production APIs and data. Changes here can affect live ShroudEdit content.',
	},
})

function hideDevBanner() {
	cosmetics.value.hideDevBanner = true
}
</script>

<template>
	<PagewideBanner v-if="flags.showAllBanners || !cosmetics.hideDevBanner" variant="warning">
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
				@click="hideDevBanner"
			>
				<XIcon aria-hidden="true" />
			</IconButton>
		</template>
	</PagewideBanner>
</template>
