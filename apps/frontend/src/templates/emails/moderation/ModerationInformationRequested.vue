<script setup lang="ts">
import { Button, Heading, Img, Link as VLink, Section, Text } from '@vue-email/components'

import StyledEmail from '../shared/StyledEmail.vue'
</script>

<template>
	<StyledEmail
		title="We need a little more information about {project.name}"
		:manual-links="[
			{
				link: 'https://shroudedit.com/project/{project.id}/moderation',
				label: 'Open moderation thread',
			},
		]"
	>
		<Section class="mb-4 mt-2">
			<Img
				src="{project.icon_url}"
				alt="project icon"
				width="64"
				height="64"
				class="block h-auto rounded-lg"
			/>
		</Section>

		<Heading as="h1" class="mb-2 text-2xl font-bold"
			>We need a little more information about {project.name}</Heading
		>

		<Text class="text-base">Hi <span class="no-auto-link">{user.name}</span>,</Text>

		<Text class="text-base">
			While reviewing version {version.name}, our moderation team found something that needs
			clarification before the review can continue.
		</Text>

		<Section class="bg-bg-super mb-4 mt-4 rounded-lg border border-divider pb-4 pl-4 pr-4 pt-4">
			<Text class="m-0 text-base">
				Please sign in and reply to the question in your moderation thread. Downloads will remain
				unavailable until the review is complete.
			</Text>
		</Section>

		<Button
			href="https://shroudedit.com/project/{project.id}/moderation"
			target="_blank"
			class="text-accentContrast bg-mail-brand inline-block rounded-[12px] pb-3 pl-4 pr-4 pt-3 text-[14px] font-bold"
		>
			Open moderation thread
		</Button>

		<Text class="text-base">
			Questions? Contact ShroudEdit Support at
			<VLink href="mailto:support@shroudedit.com" class="text-mail-brand underline"
				>support@shroudedit.com</VLink
			>.
		</Text>
	</StyledEmail>
</template>
