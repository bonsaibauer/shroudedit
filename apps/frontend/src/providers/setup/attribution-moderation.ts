import { attributionQuickReplies } from '@shroudedit/moderation'
import { provideAttributionModeration } from '@shroudedit/ui'

export function setupAttributionModerationProvider() {
	provideAttributionModeration({ attributionQuickReplies })
}
