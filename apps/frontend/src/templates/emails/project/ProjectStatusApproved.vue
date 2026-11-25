<script setup lang="ts">
import { Button, Heading, Img, Link as VLink, Section, Text } from '@vue-email/components'

import StyledEmail from '../shared/StyledEmail.vue'
</script>

<template>
	<StyledEmail
		title="Your project, {project.name}, has been approved 🎉"
		:manual-links="[
			{ link: 'https://shroudedit.com/project/{project.id}', label: 'Project page' },
			{ link: 'https://shroudedit.com/legal/rules', label: 'Community Guidelines' },
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
			>Your project, {project.name}, has been approved 🎉</Heading
		>

		<Text class="text-base">Congratulations <span class="no-auto-link">{user.name}</span>,</Text>

		<Text class="text-base">
			Your project
			<VLink href="https://shroudedit.com/project/{project.id}" class="text-mail-brand underline"
				>{project.name}</VLink
			>
			has been <b>approved</b> by the moderation team!
		</Text>

		<Button
			href="https://shroudedit.com/project/{project.id}"
			target="_blank"
			class="text-accentContrast bg-mail-brand inline-block rounded-[12px] pb-3 pl-4 pr-4 pt-3 text-[14px] font-bold"
		>
			View project
		</Button>

		<Text class="text-base">
			If you have questions or believe something isn't correct, you can contact support at
			<VLink href="mailto:support@shroudedit.com" class="text-mail-brand underline"
				>support@shroudedit.com</VLink
			>.
		</Text>

		<Text class="text-base">Thank you for sharing your work with the ShroudEdit community!</Text>
	</StyledEmail>
</template>
