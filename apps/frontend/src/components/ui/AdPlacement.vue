<template>
	<aside
		v-if="eligible"
		class="col-span-full mb-3 flex w-full flex-col items-center gap-1"
		:aria-label="formatMessage(messages.advertisement)"
	>
		<span class="text-xs text-secondary">{{ formatMessage(messages.advertisement) }}</span>
		<div class="relative max-w-full overflow-hidden rounded-xl" :style="dimensions">
			<AutoLink
				v-if="!filled"
				to="/discover/schematics"
				class="absolute inset-0 flex items-center justify-center bg-surface-3 px-3 text-center text-sm font-semibold text-primary hover:underline"
				:aria-label="formatMessage(messages.schematics)"
			>
				<template v-if="mobile">{{ formatMessage(messages.schematics) }}</template>
				<img
					v-else
					src="/news/article/welcome-to-shroudedit/thumbnail.webp"
					alt=""
					class="absolute inset-0 h-full w-full object-cover"
					loading="lazy"
				/>
			</AutoLink>
			<div
				:id="slotId"
				ref="slotElement"
				class="absolute inset-0"
				:class="{ invisible: !filled }"
				:aria-hidden="!filled"
			/>
		</div>
	</aside>
</template>

<script setup lang="ts">
import { AutoLink, defineMessages, useVIntl } from '@shroudedit/ui'
import { computed, nextTick, onBeforeUnmount, onMounted, ref, useId, watch } from 'vue'

import { useAuthState } from '~/composables/auth'
import { useAdConsent } from '~/composables/useAdConsent'
import { type GamSlot, registerGamSlot } from '~/utils/google-publisher-tag'

const config = useRuntimeConfig().public
const props = defineProps<{ placement: keyof typeof config.gamAdUnits }>()
const auth = useAuthState()
const route = useRoute()
const { formatMessage } = useVIntl()
const { canRequestAds, initializeConsent } = useAdConsent()
const messages = defineMessages({
	advertisement: { id: 'ads.placement.label', defaultMessage: 'Advertisement' },
	schematics: { id: 'ads.placement.schematics', defaultMessage: 'Explore Enshrouded schematics' },
})

const slotId = `ad-${props.placement}-${useId()}`
const slotElement = ref<HTMLElement>()
const filled = ref(false)
const fitsViewport = ref(false)
const mobile = computed(() => props.placement.endsWith('Mobile'))
const size = computed<[number, number]>(() => (mobile.value ? [320, 50] : [300, 250]))
const dimensions = computed(() => ({ width: `${size.value[0]}px`, height: `${size.value[1]}px` }))
const path = computed(() => config.gamAdUnits[props.placement].trim())
const eligible = computed(
	() =>
		config.adsEnabled &&
		!auth.value.user &&
		fitsViewport.value &&
		/^\/\d+\/[^\s]+$/.test(path.value),
)

let viewport: MediaQueryList | undefined
function updateViewport() {
	fitsViewport.value = viewport?.matches === true
}

onMounted(() => {
	if (!config.adsEnabled) return
	viewport = window.matchMedia(
		mobile.value ? '(min-width: 320px) and (max-width: 1024px)' : '(width > 1024px)',
	)
	viewport.addEventListener('change', updateViewport)
	updateViewport()
})

const stopPlacement = watch(
	[eligible, () => route.path, path],
	([active, , adUnitPath], _, onCleanup) => {
		if (!active) return
		const abort = new AbortController()
		let observer: IntersectionObserver | undefined
		let resizeObserver: ResizeObserver | undefined
		let slot: GamSlot | undefined
		let registering = false
		let requested = false
		let visible = false

		function clearSlot() {
			slot?.destroy()
			slot = undefined
			requested = false
			filled.value = false
		}

		function requestWhenReady() {
			if (abort.signal.aborted) return
			const fitsContainer = (slotElement.value?.clientWidth ?? 0) >= size.value[0]
			if (slot && visible && fitsContainer && canRequestAds.value && !requested) {
				requested = true
				slot.request()
			}
		}

		async function register(element: HTMLElement) {
			if (registering || slot || abort.signal.aborted) return
			registering = true
			try {
				slot = await registerGamSlot(element, adUnitPath, size.value, abort.signal, (hasAd) => {
					filled.value = hasAd
				})
				if (abort.signal.aborted) return clearSlot()
				requestWhenReady()
			} catch {
				return
			} finally {
				registering = false
			}
		}

		const stopConsent = watch(
			canRequestAds,
			(allowed) => {
				if (!allowed && requested) clearSlot()
				if (allowed && slotElement.value) {
					void register(slotElement.value)
					requestWhenReady()
				}
			},
			{ flush: 'sync' },
		)

		onCleanup(() => {
			abort.abort()
			stopConsent()
			observer?.disconnect()
			resizeObserver?.disconnect()
			clearSlot()
		})

		void nextTick(() => {
			const element = slotElement.value
			if (abort.signal.aborted || !element) return
			initializeConsent()
			observer = new IntersectionObserver(([entry]) => {
				visible = entry?.isIntersecting === true
				requestWhenReady()
			})
			observer.observe(element)
			resizeObserver = new ResizeObserver(requestWhenReady)
			resizeObserver.observe(element)
			void register(element)
		})
	},
	{ flush: 'pre' },
)

onBeforeUnmount(() => {
	stopPlacement()
	viewport?.removeEventListener('change', updateViewport)
})
</script>
