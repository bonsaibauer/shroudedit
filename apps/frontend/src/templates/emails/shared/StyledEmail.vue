<script setup lang="ts">
import { Column, Container, Link as VLink, Row, Section, Text } from '@vue-email/components'

import StyledTemplate from '../../shared/StyledTemplate.vue'

defineProps<{
	title?: string
	manualLinks?: { link: string; label?: string }[]
	supportInfo?: string[]
}>()

</script>

<template>
	<StyledTemplate :title="title">
		<Section class="bg-bg pb-4 pl-4 pr-4 pt-4">
			<Container class="max-w-[600px]">
				<Row>
					<Column>
						<VLink href="https://shroudedit.com" class="text-xl font-bold text-green">
							ShroudEdit
						</VLink>
					</Column>
				</Row>
			</Container>
		</Section>

		<Section class="bg-white pb-8 pl-8 pr-8 pt-8">
			<Container class="max-w-[600px]">
				<slot />
			</Container>
		</Section>

		<Section class="mb-4 bg-bg pb-4 pl-4 pr-4 pt-4">
			<Container class="max-w-[600px]">
				<Row>
					<Column class="align-middle">
						<VLink href="https://shroudedit.com" aria-label="ShroudEdit" class="font-bold text-green">
							ShroudEdit
						</VLink>
						<Text class="mb-0 mt-2 text-xs" :style="{ color: '#4d4d4d' }">
							Enshrouded mods, schematics, modpacks and servers.
						</Text>
					</Column>
				</Row>
			</Container>
		</Section>

		<!-- <Text
			class="text-footerText text-2xs mb-4 mt-0 pb-0 pl-4 pr-4 pt-0 text-center font-sans"
		>
			This email was sent to you as a registered user of ShroudEdit. You can customize the
			emails you recieve in your
			<VLink href="https://shroudedit.com/settings/notifications" class="text-green underline"
				>notification settings</VLink
			>. Some emails are required to keep your account secure and cannot be disabled.
		</Text> -->

		<hr />

		<Section v-if="supportInfo && supportInfo.length" class="mb-0 pb-0 pl-4 pr-4 pt-0">
			<Text
				v-for="(line, index) in supportInfo"
				:key="index"
				class="text-footerText text-2xs font-sans"
			>
				{{ line }}
			</Text>
		</Section>

		<Section
			v-if="manualLinks && manualLinks.length"
			class="text-footerText text-2xs mb-4 pb-0 pl-4 pr-4 pt-0 font-sans"
		>
			<small class="text-muted text-2xs"
				>If you're having trouble with the links above, copy and paste these URLs into your web
				browser:</small
			>
			<Text class="text-2xs text-muted mt-0">
				<span v-for="(item, index) in manualLinks" :key="index" class="block break-words">
					<span v-if="item.label">
						<b>{{ item.label }}:</b><br />
					</span>
					{{ item.link }}
				</span>
				<!-- <span class="block break-words">
					<span> <b>Notification settings:</b><br /> </span>
					https://shroudedit.com/settings/notifications
				</span> -->
			</Text>
		</Section>
	</StyledTemplate>
</template>
