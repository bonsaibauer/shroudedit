interface TcfData {
	gdprApplies?: boolean
	cmpStatus?: string
	eventStatus?: string
	tcString?: string
	purpose?: { consents?: Record<number, boolean> }
	vendor?: { consents?: Record<number, boolean> }
}

type TcfApi = (
	command: 'addEventListener',
	version: number,
	callback: (data: TcfData, success: boolean) => void,
) => void

interface FundingChoices {
	callbackQueue?: { push(callback: { CONSENT_API_READY: () => void }): void }
	showRevocationMessage?: () => void
}

type ConsentWindow = Window & {
	__tcfapi?: TcfApi
	googlefc?: FundingChoices
}

const GOOGLE_TCF_VENDOR = 755
const PERSONALIZED_AD_PURPOSES = [1, 3, 4]
let listening = false

export function useAdConsent() {
	const canRequestAds = useState('ads:can-request', () => false)
	const canManageConsent = useState('ads:can-manage-consent', () => false)

	function initializeConsent() {
		if (listening) return
		listening = true
		const browser = window as ConsentWindow
		const fundingChoices = (browser.googlefc ??= {})
		const queue: NonNullable<FundingChoices['callbackQueue']> = (fundingChoices.callbackQueue ??=
			[])
		queue.push({
			CONSENT_API_READY: () => {
				browser.__tcfapi?.('addEventListener', 0, (data, success) => {
					const applies = success && data?.gdprApplies === true
					canManageConsent.value =
						applies && typeof browser.googlefc?.showRevocationMessage === 'function'
					const decisionReady =
						data?.eventStatus === 'tcloaded' || data?.eventStatus === 'useractioncomplete'
					canRequestAds.value =
						success &&
						data?.cmpStatus !== 'error' &&
						(data?.gdprApplies === false ||
							(applies &&
								decisionReady &&
								!!data.tcString &&
								data.vendor?.consents?.[GOOGLE_TCF_VENDOR] === true &&
								PERSONALIZED_AD_PURPOSES.every(
									(purpose) => data.purpose?.consents?.[purpose] === true,
								)))
				})
			},
		})
	}

	function manageAdConsent() {
		const fundingChoices = (window as ConsentWindow).googlefc
		if (!fundingChoices?.showRevocationMessage) return
		canRequestAds.value = false
		fundingChoices.showRevocationMessage()
	}

	return { canRequestAds, canManageConsent, initializeConsent, manageAdConsent }
}
