import dayjs from 'dayjs'

export type Product = 'web' | 'hosting' | 'app'

export type VersionEntry = {
	date: dayjs.Dayjs
	product: Product
	version?: string
	body: string
}

const VERSIONS: VersionEntry[] = [
	{
		date: dayjs('2026-09-20T00:00:00.000Z'),
		product: 'web',
		body: `## Added
- Added ShroudEdit support for Enshrouded mods, schematics, modpacks, and community servers.
- Added Shroudtopia and Shroudforge loader metadata.
- Added Enshrouded dedicated-server status and access information.`,
	},
]

export function getChangelog() {
	return VERSIONS
}
