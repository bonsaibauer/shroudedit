export default defineEventHandler((event) => {
	const entries = useRuntimeConfig(event).adsTxt.replaceAll('\\n', '\n').trim()
	if (!entries) throw createError({ statusCode: 404 })

	setHeader(event, 'Content-Type', 'text/plain; charset=utf-8')
	return `${entries}\n`
})
