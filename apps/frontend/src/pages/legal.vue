<template>
	<div class="normal-page">
		<div class="normal-page__sidebar">
			<NavStack
				:items="[
					{ type: 'heading', label: 'Legal' },
					{ link: '/legal/imprint', label: 'Impressum', icon: InfoIcon },
					{ link: '/legal/rules', label: 'Content Rules', icon: ScaleIcon },
					{ link: '/legal/terms', label: 'Terms of Use', icon: HeartHandshakeIcon },
					{ link: '/legal/privacy', label: 'Privacy Policy', icon: LockIcon },
					{ link: '/legal/security', label: 'Security Notice', icon: ShieldIcon },
					{ link: '/legal/copyright', label: 'Copyright Policy', icon: CopyrightIcon },
				]"
			/>
		</div>
		<div class="normal-page__content">
			<NuxtPage class="universal-card" :route="route" />
		</div>
	</div>
</template>

<script setup>
import {
	CopyrightIcon,
	HeartHandshakeIcon,
	InfoIcon,
	LockIcon,
	ScaleIcon,
	ShieldIcon,
} from '@shroudedit/assets'

import NavStack from '~/components/ui/NavStack.vue'

const route = useNativeRoute()
</script>

<style lang="scss" scoped>
.normal-page__content :deep(a) {
	color: var(--color-link);
	text-decoration: underline;

	&:focus-visible,
	&:hover {
		color: var(--color-link-hover);
	}

	&:active {
		color: var(--color-link-active);
	}
}
</style>
