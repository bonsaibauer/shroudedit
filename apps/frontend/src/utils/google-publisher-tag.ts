type AdSize = [number, number]

interface Slot {
	addService(service: PubAdsService): Slot
}

interface SlotRenderEndedEvent {
	slot: Slot
	isEmpty: boolean
}

interface PubAdsService {
	addEventListener(name: 'slotRenderEnded', listener: (event: SlotRenderEndedEvent) => void): void
	removeEventListener(
		name: 'slotRenderEnded',
		listener: (event: SlotRenderEndedEvent) => void,
	): void
	refresh(slots: Slot[]): void
}

interface PublisherTag {
	cmd: { push(callback: () => void): number }
	setConfig(config: {
		disableInitialLoad: boolean
		autoRefresh: { backForwardCache: boolean; heavyAds: boolean }
	}): void
	defineSlot(path: string, size: AdSize, id: string): Slot | null
	pubads(): PubAdsService
	enableServices(): void
	display(element: HTMLElement): void
	destroySlots(slots: Slot[]): boolean
}

export interface GamSlot {
	request(): void
	destroy(): void
}

let tagPromise: Promise<PublisherTag> | undefined
let servicesEnabled = false

function loadPublisherTag(): Promise<PublisherTag> {
	if (tagPromise) return tagPromise

	tagPromise = new Promise((resolve, reject) => {
		const browser = window as Window & { googletag?: Pick<PublisherTag, 'cmd'> }
		const queue = (browser.googletag ??= { cmd: [] })
		queue.cmd.push(() => {
			const tag = browser.googletag as PublisherTag
			tag.setConfig({
				disableInitialLoad: true,
				autoRefresh: { backForwardCache: false, heavyAds: false },
			})
			resolve(tag)
		})

		const script = document.createElement('script')
		script.src = 'https://securepubads.g.doubleclick.net/tag/js/gpt.js'
		script.async = true
		script.crossOrigin = 'anonymous'
		script.addEventListener('error', reject, { once: true })
		document.head.append(script)
	})

	return tagPromise
}

export async function registerGamSlot(
	element: HTMLElement,
	path: string,
	size: AdSize,
	signal: AbortSignal,
	onRender: (filled: boolean) => void,
): Promise<GamSlot | undefined> {
	const tag = await loadPublisherTag()
	if (signal.aborted || !element.isConnected) return

	const slot = tag.defineSlot(path, size, element.id)
	if (!slot) return

	const service = tag.pubads()
	slot.addService(service)
	if (!servicesEnabled) {
		tag.enableServices()
		servicesEnabled = true
	}

	function onSlotRender(event: SlotRenderEndedEvent) {
		if (event.slot === slot) onRender(!event.isEmpty)
	}

	service.addEventListener('slotRenderEnded', onSlotRender)
	tag.display(element)

	return {
		request: () => service.refresh([slot]),
		destroy() {
			service.removeEventListener('slotRenderEnded', onSlotRender)
			tag.destroySlots([slot])
		},
	}
}
