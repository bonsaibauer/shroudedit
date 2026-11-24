<script setup lang="ts">
import { Heading, Link as VLink, Text } from '@vue-email/components'

import StyledEmail from '../shared/StyledEmail.vue'
</script>

<template>
	<StyledEmail title="Your email has been changed">
		<Heading as="h1" class="mb-2 text-2xl font-bold"> Your email has been changed </Heading>

		<Text class="text-muted text-base">Hi <span class="no-auto-link">{user.name}</span>,</Text>
		<Text class="text-muted text-base">
			At your request, we've successfully updated your ShroudEdit account's email to
			{emailchanged.new_email}.
		</Text>
		<Text class="text-muted text-base">
			If you did not make this change, please contact us immediately at
			<VLink href="mailto:support@shroudedit.com" class="text-mail-brand underline"
				>support@shroudedit.com</VLink
			>.
		</Text>
	</StyledEmail>
</template>
