<script setup lang="ts">
import { Heading, Link as VLink, Text } from '@vue-email/components'

import StyledEmail from '../shared/StyledEmail.vue'
</script>

<template>
	<StyledEmail title="A new personal access token has been created">
		<Heading as="h1" class="mb-2 text-2xl font-bold">
			A new personal access token has been created
		</Heading>

		<Text class="text-muted text-base">Hi <span class="no-auto-link">{user.name}</span>,</Text>
		<Text class="text-muted text-base">
			A new personal access token, <b>{newpat.token_name}</b>, has been added to your account.
		</Text>
		<Text class="text-muted text-base">
			If you did not create this token, please contact us immediately at
			<VLink href="mailto:support@shroudedit.com" class="text-mail-brand underline"
				>support@shroudedit.com</VLink
			>.
		</Text>
	</StyledEmail>
</template>
