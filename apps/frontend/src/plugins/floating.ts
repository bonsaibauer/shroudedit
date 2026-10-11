import { installTooltipDirective } from '@shroudedit/ui'

export default defineNuxtPlugin((nuxtApp) => {
	installTooltipDirective(nuxtApp.vueApp)
})
