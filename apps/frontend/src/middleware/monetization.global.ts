const disabledPaths = [
	'/admin/affiliates',
	'/dashboard/affiliate-links',
	'/dashboard/revenue',
	'/legal/cmp-info',
]

export default defineNuxtRouteMiddleware((to) => {
	if (useRuntimeConfig().public.monetizationEnabled) return
	if (!disabledPaths.some((path) => to.path === path || to.path.startsWith(`${path}/`))) return

	return navigateTo(to.path.startsWith('/dashboard') ? '/dashboard' : '/')
})
