<script setup lang="ts">
import { Heading, Text } from '@vue-email/components'

import StyledEmail from '../shared/StyledEmail.vue'
</script>

<template>
	<StyledEmail title="Your subscription tax is changing">
		<Heading as="h1" class="mb-2 text-2xl font-bold">Your subscription tax is changing</Heading>

		<Text class="text-muted text-base">Hi <span class="no-auto-link">{user.name}</span>,</Text>

		<Text class="text-muted text-base">
			The tax rate for your {taxnotification.service} subscription is changing. Starting with
			your next {taxnotification.billing_interval} payment, the charge will be updated as follows.
		</Text>

		<Text class="text-muted text-base">
			Current subtotal: {taxnotification.old_amount}<br />
			Current tax: {taxnotification.old_tax_amount}<br />
			Current total: {taxnotification.old_total_amount}
		</Text>

		<Text class="text-muted text-base">
			New subtotal: {taxnotification.new_amount}<br />
			New tax: {taxnotification.new_tax_amount}<br />
			New total: {taxnotification.new_total_amount}
		</Text>

		<Text class="text-muted text-base">
			The pre-tax price of your subscription has not changed. Only the tax amount has changed
			to reflect local tax requirements.
		</Text>
	</StyledEmail>
</template>
