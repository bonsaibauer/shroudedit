import { ref } from 'vue'

import { injectAppBackup } from '#ui/providers'

export function useInlineBackup() {
	const appBackup = injectAppBackup(null)

	if (appBackup) {
		const isBackingUp = ref(false)
		const backupFailed = ref(false)
		const backupComplete = ref(false)

		return {
			available: true as const,
			isBackingUp,
			backupFailed,
			backupComplete,
			startBackup: async () => {
				isBackingUp.value = true
				backupFailed.value = false
				backupComplete.value = false
				try {
					await appBackup.createBackup()
					backupComplete.value = true
				} catch {
					backupFailed.value = true
				} finally {
					isBackingUp.value = false
				}
			},
			cancelBackup: async () => {},
		}
	}

	return {
		available: false as const,
		isBackingUp: ref(false),
		backupFailed: ref(false),
		backupComplete: ref(false),
		startBackup: async () => {},
		cancelBackup: async () => {},
	}
}
