export const VISITOR_USER_AGENT_HEADER = 'X-Forwarded-User-Agent'

export function getFrontendUserAgent(commitHash: string): string {
	return `shroudedit/frontend/${commitHash || 'unknown'} (support@shroudedit.com)`
}
