<script setup lang="ts">
import { Heading, Link as VLink, Text } from '@vue-email/components'

import StyledEmail from '../shared/StyledEmail.vue'
</script>

<template>
	<StyledEmail title="Sign-in method removed">
		<Heading as="h1" class="mb-2 text-2xl font-bold"> Sign-in method removed</Heading>

		<Text class="text-muted text-base">Hi <span class="no-auto-link">{user.name}</span>,</Text>
		<Text class="text-muted text-base">
			Your <b>{authprovider.name}</b> account has been disconnected and you can no longer use it to
			sign in to your ShroudEdit account.
		</Text>
		<Text class="text-muted text-base">
			If you did not make this change, please contact us immediately at
			<VLink href="mailto:support@shroudedit.com" class="text-mail-brand underline"
				>support@shroudedit.com</VLink
			>.
		</Text>
	</StyledEmail>
</template>
