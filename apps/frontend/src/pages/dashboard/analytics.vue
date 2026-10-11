<template>
	<AnalyticsDashboard />
</template>

<script setup>
import { commonProjectSettingsMessages, useVIntl } from '@shroudedit/ui'

import AnalyticsDashboard from '~/components/analytics-dashboard/index.vue'

const { formatMessage } = useVIntl()

definePageMeta({
	middleware: 'auth',
})

useHead({
	title: () => `${formatMessage(commonProjectSettingsMessages.analytics)} - ShroudEdit`,
})
</script>
