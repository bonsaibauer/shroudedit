<template>
	<div class="normal-page">
		<div class="normal-page__sidebar">
			<NavStack
				:items="[
					{ type: 'heading', label: formatMessage(legalMessages.heading) },
					{ link: '/legal/imprint', label: formatMessage(legalMessages.imprint), icon: InfoIcon },
					{ link: '/legal/rules', label: formatMessage(legalMessages.rules), icon: ScaleIcon },
					{
						link: '/legal/terms',
						label: formatMessage(legalMessages.terms),
						icon: HeartHandshakeIcon,
					},
					{ link: '/legal/privacy', label: formatMessage(legalMessages.privacy), icon: LockIcon },
					{
						link: '/legal/security',
						label: formatMessage(legalMessages.security),
						icon: ShieldIcon,
					},
					{
						link: '/legal/copyright',
						label: formatMessage(legalMessages.copyright),
						icon: CopyrightIcon,
					},
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
import { useVIntl } from '@shroudedit/ui'

import NavStack from '~/components/ui/NavStack.vue'
import { legalMessages } from '~/utils/legal-messages'

const { formatMessage } = useVIntl()
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
