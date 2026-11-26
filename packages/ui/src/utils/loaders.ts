export const loaderDisplayNames: Record<string, string> = {
	shroudforge: 'ShroudForge',
	eml: 'EML',
	shroudtopia: 'Shroudtopia',
	sfpack: 'ShroudForge Pack',
}

export const formatLoaderLabel = (item: string) =>
	loaderDisplayNames[item] ?? item.charAt(0).toUpperCase() + item.slice(1)
