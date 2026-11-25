<script setup lang="ts">
import { Button, Heading, Link as VLink, Text } from '@vue-email/components'

import StyledEmail from '../shared/StyledEmail.vue'
</script>

<template>
	<StyledEmail
		title="Revenue is available to withdraw"
		:manual-links="[
			{ link: 'https://shroudedit.com/dashboard/revenue', label: 'Revenue dashboard' },
		]"
	>
		<Heading as="h1" class="mb-2 text-2xl font-bold">Revenue is available to withdraw</Heading>

		<Text class="text-muted text-base">Hi <span class="no-auto-link">{user.name}</span>,</Text>

		<Text class="text-muted text-base">
			The {payout.amount} earned during {payout.period} has been processed and is now available to
			withdraw from your account.
		</Text>

		<Button
			href="https://shroudedit.com/dashboard/revenue"
			target="_blank"
			class="text-accentContrast bg-mail-brand inline-block rounded-[12px] pb-3 pl-4 pr-4 pt-3 text-[14px] font-bold"
		>
			View revenue dashboard
		</Button>

		<Text class="text-muted text-base">
			If you have questions about creator payouts, contact
			<VLink href="mailto:support@shroudedit.com" class="text-mail-brand underline"
				>support@shroudedit.com</VLink
			>.
		</Text>
	</StyledEmail>
</template>
