<template>
	<div class="shadow-card rounded-2xl border border-solid border-surface-4 bg-surface-3 p-4">
		<div class="flex flex-wrap items-center justify-between gap-3">
			<div class="flex min-w-0 items-center gap-4">
				<NuxtLink :to="`/project/${projectRouteParam}`" target="_blank" tabindex="-1" class="flex">
					<Avatar
						:src="project.icon_url"
						size="4rem"
						class="rounded-2xl border border-surface-5 bg-surface-4 !shadow-none"
					/>
				</NuxtLink>
				<div class="flex min-w-0 flex-col gap-1.5">
					<div class="flex flex-wrap items-center gap-2">
						<NuxtLink
							:to="`/project/${projectRouteParam}`"
							target="_blank"
							class="min-w-0 truncate text-lg font-semibold text-contrast hover:underline"
						>
							{{ project.name }}
						</NuxtLink>
						<div
							class="flex items-center gap-1 rounded-full border border-solid border-surface-5 bg-surface-4 px-2.5 py-1"
						>
							<component
								:is="getProjectTypeIcon(primaryType as any)"
								aria-hidden="true"
								class="size-4"
							/>
							<span class="text-sm font-medium text-secondary">
								{{
									project.project_types.length === 0
										? '???'
										: project.project_types
												.map((type) =>
													formatProjectType(type === 'enshrouded_server' ? 'server' : type, true),
												)
												.join(', ')
								}}
							</span>
						</div>
						<slot name="badges" />
					</div>
					<div v-if="ownership?.kind === 'user'">
						<NuxtLink
							:to="`/user/${ownership.id}`"
							target="_blank"
							class="flex w-fit min-w-40 items-center gap-1 text-sm font-medium text-secondary hover:underline"
						>
							<Avatar
								:src="ownership.icon_url"
								size="1.5rem"
								circle
								class="border border-surface-5 bg-surface-4 !shadow-none"
							/>
							{{ ownership.name }}
						</NuxtLink>
					</div>
					<div v-else-if="ownership?.kind === 'organization'" class="flex items-center gap-1">
						<Avatar
							:src="ownership.icon_url"
							size="1.5rem"
							class="border border-surface-5 bg-surface-4 !shadow-none"
						/>
						<NuxtLink
							:to="`/organization/${ownership.id}`"
							target="_blank"
							class="text-sm font-medium text-secondary hover:underline"
						>
							{{ ownership.name }}
						</NuxtLink>
					</div>
					<slot name="details" />
				</div>
			</div>
			<div class="flex flex-wrap items-center gap-3">
				<slot name="summary" />
				<div class="flex items-center gap-2">
					<CopyCode v-tooltip="'Copy project ID'" :text="project.id" />
					<CopyLinkButton
						v-tooltip="'Copy project link'"
						:url="`${config.public.siteUrl}/project/${project.id}`"
					/>
					<ButtonLink
						v-tooltip="'Open moderation thread'"
						:href="`/project/${project.id}/moderation`"
						target="_blank"
						circular
						icon-only
					>
						<ExternalIcon />
					</ButtonLink>
					<slot name="actions" />
				</div>
			</div>
		</div>
	</div>
</template>

<script setup lang="ts">
import { ExternalIcon } from '@shroudedit/assets'
import { Avatar, ButtonLink, CopyCode, CopyLinkButton, getProjectTypeIcon } from '@shroudedit/ui'
import { formatProjectType } from '@shroudedit/utils'

const config = useRuntimeConfig()
const props = defineProps<{
	project: {
		id: string
		name: string
		icon_url: string | null
		project_types: string[]
		slug?: string | null
	}
	ownership: {
		kind: 'user' | 'organization'
		id: string
		name: string
		icon_url?: string | null
	} | null
}>()

const projectRouteParam = computed(() => props.project.slug || props.project.id)
const primaryType = computed(() => {
	const type = props.project.project_types[0]
	return type === 'enshrouded_server' ? 'server' : type
})
</script>
