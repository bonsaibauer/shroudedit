import type { ContentItem, ManagedContentSummaryItem, ManagedContentSummaryType } from '../types'

const summaryOrder: ManagedContentSummaryType[] = ['mod', 'modpack']

function normalizeSummaryType(type: string): ManagedContentSummaryType | null {
	return summaryOrder.includes(type as ManagedContentSummaryType)
		? (type as ManagedContentSummaryType)
		: null
}

export function dedupeManagedContentItems(items: ContentItem[]): ContentItem[] {
	const seenKeys = new Set<string>()
	return items.filter((item) => {
		const key = item.file_path
			? `path:${item.file_path}`
			: item.file_name
				? `name:${item.file_name}`
				: `id:${item.id}`
		if (seenKeys.has(key)) return false

		seenKeys.add(key)
		return true
	})
}

export function summarizeManagedContent(items: ContentItem[]): ManagedContentSummaryItem[] {
	const counts = new Map<ManagedContentSummaryType, number>()
	for (const item of dedupeManagedContentItems(items)) {
		const type = normalizeSummaryType(item.project_type)
		if (type) counts.set(type, (counts.get(type) ?? 0) + 1)
	}

	return summaryOrder.flatMap((type) => {
		const count = counts.get(type) ?? 0
		return count > 0 ? [{ type, count }] : []
	})
}
