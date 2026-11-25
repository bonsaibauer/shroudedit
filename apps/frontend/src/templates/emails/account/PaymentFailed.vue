<script setup lang="ts">
import { Button, Heading, Link as VLink, Text } from '@vue-email/components'

import StyledEmail from '../shared/StyledEmail.vue'
</script>

<template>
	<StyledEmail
		title="Payment failed for ShroudEdit"
		:manual-links="[{ link: '{billing.url}', label: 'Update billing settings' }]"
	>
		<Heading as="h1" class="mb-2 text-2xl font-bold">Payment failed</Heading>

		<Text class="text-muted text-base">Hi <span class="no-auto-link">{user.name}</span>,</Text>

		<Text class="text-muted text-base">
			We could not collect {paymentfailed.amount} for your ShroudEdit subscription. Please update
			your billing details to avoid service interruption.
		</Text>

		<Button
			href="{billing.url}"
			target="_blank"
			class="text-accentContrast bg-mail-brand inline-block rounded-[12px] pb-3 pl-4 pr-4 pt-3 text-[14px] font-bold"
		>
			Update billing settings
		</Button>

		<Text class="text-muted text-base">
			If you have questions, contact
			<VLink href="mailto:support@shroudedit.com" class="text-mail-brand underline"
				>support@shroudedit.com</VLink
			>.
		</Text>
	</StyledEmail>
</template>
