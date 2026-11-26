import type { Labrinth } from '@shroudedit/api-client'

import { inferSfbpMetadata } from './sfbp'
import { inferSfpackVersion, InvalidSfpackError } from './sfpack'
import { inferShroudForgeVersion } from './shroudforge'

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
> & { mod_id?: string; package_fields?: string[] }

export async function inferVersionInfo(
	file: File,
	project: Pick<Labrinth.Projects.v2.Project, 'actualProjectType'>,
): Promise<InferredVersionInfo> {
	const filename = file.name.toLowerCase()
	if (project.actualProjectType === 'schematic') {
		if (!filename.endsWith('.sfbp')) {
			throw new Error('Blueprints must use the .sfbp file extension')
		}
		return inferSfbpMetadata(file)
	}

	const isModpack = filename.endsWith('.sfpack')
	if (project.actualProjectType === 'modpack' && !isModpack) {
		throw new InvalidSfpackError('Modpacks must use the .sfpack file extension.')
	}
	if (isModpack && file.size > 2 * 1024 * 1024) {
		throw new InvalidSfpackError('The .sfpack archive exceeds 2 MiB.')
	}
	if (!isModpack && !filename.endsWith('.zip')) {
		return {}
	}

	const { BlobReader, ZipReader } = await import('@zip.js/zip.js')
	const reader = new ZipReader(new BlobReader(file))
	try {
		const entries = await reader.getEntries().catch((error) => {
			if (isModpack) throw new InvalidSfpackError('The .sfpack archive is not a valid ZIP.')
			throw error
		})
		if (!isModpack) {
			const inferred = await inferShroudForgeVersion(entries.filter((entry) => !entry.directory))
			return inferred
		}
		return inferSfpackVersion(entries.filter((entry) => !entry.directory))
	} finally {
		await reader.close()
	}
}
