export default defineNuxtRouteMiddleware((to) => {
	if (
		to.path.startsWith('/mods') ||
		to.path.startsWith('/modpacks') ||
		to.path.startsWith('/schematics')
	) {
		const target = '/discover' + to.fullPath
		return navigateTo(target, { redirectCode: 301 })
	}
})
