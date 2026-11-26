export const PROJECT_TYPE_ORDER = [
	'mod',
	'modpack',
	'schematic',
	'server',
	'collection',
] as const

export type OrderedProjectType = (typeof PROJECT_TYPE_ORDER)[number]

export function getProjectTypeSortIndex(type: string): number {
	const index = (PROJECT_TYPE_ORDER as readonly string[]).indexOf(type)
	return index === -1 ? PROJECT_TYPE_ORDER.length : index
}

export function compareProjectTypes(a: string, b: string): number {
	return getProjectTypeSortIndex(a) - getProjectTypeSortIndex(b)
}

export function sortProjectTypes<T extends string>(types: Iterable<T>): T[] {
	return [...types].sort(compareProjectTypes)
}
