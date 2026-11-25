<script setup lang="ts">
import { Heading, Text } from '@vue-email/components'

import StyledEmail from '../shared/StyledEmail.vue'
</script>

<template>
	<StyledEmail title="Time has been added to your subscription">
		<Heading as="h1" class="mb-2 text-2xl font-bold">
			Time has been added to your subscription
		</Heading>

		<Text class="text-muted text-base">Hi <span class="no-auto-link">{user.name}</span>,</Text>

		<Text class="text-muted text-base">
			{credit.header_message}
		</Text>

		<Text class="text-muted text-base">
			We have added {credit.days_formatted} to your {credit.subscription.type} subscription.
		</Text>

		<Text class="text-muted text-base">
			Your next charge was scheduled for {credit.previous_due} and will now be on
			{credit.next_due}.
		</Text>

		<Text class="text-muted text-base">Thank you for supporting ShroudEdit.</Text>
	</StyledEmail>
</template>
