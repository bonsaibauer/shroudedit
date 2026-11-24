import type { Labrinth } from '@shroudedit/api-client'

import { inferShroudForgeVersion, InvalidModManifestError } from './shroudforge'

export type InferredVersionInfo = Partial<
	Pick<
		Labrinth.Versions.v3.DraftVersion,
		| 'name'
		| 'version_number'
		| 'version_type'
		| 'loaders'
		| 'game_versions'
		| 'environment'
		| 'changelog'
		| 'schematic_format_version'
		| 'schematic_width'
		| 'schematic_height'
		| 'schematic_depth'
	>
>

function versionType(version: string): Labrinth.Versions.v3.VersionChannel {
	const prerelease = version.split('+')[0].split('-').slice(1).join('-')
	if (prerelease.startsWith('alpha')) return 'alpha'
	return prerelease ? 'beta' : 'release'
}

async function inferSfbpMetadata(
	rawFile: File,
): Promise<
	Pick<
		InferredVersionInfo,
		'schematic_format_version' | 'schematic_width' | 'schematic_height' | 'schematic_depth'
	>
> {
	const content = await rawFile.slice(0, 1_048_576).text()
	const lines = content.split(/\r?\n/)
	if (lines[0] !== 'SHROUDFORGE_WORLD_BLUEPRINT_V7') {
		throw new Error('Blueprints must use the ShroudForge SFBP V7 format')
	}
	if (!['x', 'y', 'z'].includes(lines[1] ?? '')) {
		throw new Error('SFBP rotation axis must be x, y, or z')
	}
	const extent = (lines[2] ?? '').split(',').map(Number)
	if (extent.length !== 3 || extent.some((value) => !Number.isFinite(value) || value < 0)) {
		throw new Error('SFBP extent must contain three non-negative numbers')
	}

	let dimensions: number[]
	if (lines[3] === 'voxel') {
		dimensions = (lines[4] ?? '').split(',').map(Number)
		const cellCount = dimensions.reduce((total, dimension) => total * dimension, 1)
		if (
			dimensions.length !== 3 ||
			dimensions.some((value) => !Number.isInteger(value) || value < 1 || value > 65_536) ||
			!Number.isSafeInteger(cellCount) ||
			cellCount > 65_536
		) {
			throw new Error('SFBP voxel dimensions exceed the supported limit')
		}
	} else if (lines[3] === 'props') {
		if (extent.some((value) => value <= 0)) {
			throw new Error('Props-only SFBP extent must be greater than zero')
		}
		dimensions = extent.map(Math.ceil)
	} else {
		throw new Error('SFBP blueprint kind must be voxel or props')
	}
	if (dimensions.some((value) => value > 65_536)) {
		throw new Error('SFBP dimensions exceed the supported limit')
	}

	return {
		schematic_format_version: 7,
		schematic_width: dimensions[0],
		schematic_height: dimensions[1],
		schematic_depth: dimensions[2],
	}
}

export async function inferVersionInfo(
	file: File,
	project: Pick<Labrinth.Projects.v2.Project, 'title' | 'actualProjectType'>,
	gameVersions: readonly Labrinth.Tags.v2.GameVersion[],
): Promise<InferredVersionInfo> {
	const filename = file.name.toLowerCase()
	if (project.actualProjectType === 'schematic') {
		if (!filename.endsWith('.sfbp')) {
			throw new Error('Blueprints must use the .sfbp file extension')
		}
		return inferSfbpMetadata(file)
	}

	const isModpack = filename.endsWith('.mrpack') || filename.endsWith('.mrpack-primary')
	const requireManifest =
		!isModpack && (project.actualProjectType === 'mod' || project.actualProjectType === 'project')
	if (!isModpack && !filename.endsWith('.zip')) {
		if (requireManifest) throw new InvalidModManifestError()
		return {}
	}

	const { BlobReader, TextWriter, ZipReader } = await import('@zip.js/zip.js')
	const reader = new ZipReader(new BlobReader(file))
	try {
		const entries = await reader.getEntries().catch((error) => {
			if (requireManifest) throw new InvalidModManifestError()
			throw error
		})
		if (!isModpack) {
			const inferred = await inferShroudForgeVersion(
				entries.filter((entry) => !entry.directory),
				requireManifest,
			)
			if (inferred.version_number) inferred.version_type = versionType(inferred.version_number)
			return inferred
		}

		const manifest = entries.find(
			(entry) => !entry.directory && entry.filename === 'modrinth.index.json',
		)
		if (!manifest || manifest.directory) throw new Error('Missing modrinth.index.json')

		const pack: { versionId?: string; dependencies?: { enshrouded?: string } } = JSON.parse(
			await manifest.getData(new TextWriter()),
		)
		const version = pack.versionId
		const gameVersion = pack.dependencies?.enshrouded
		return {
			name: version ? `${project.title} ${version}` : undefined,
			version_number: version,
			version_type: version ? versionType(version) : undefined,
			game_versions: gameVersions
				.filter(({ version }) => version === gameVersion)
				.map(({ version }) => version),
		}
	} finally {
		await reader.close()
	}
}
