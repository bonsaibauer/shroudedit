import { type FileEntry, TextWriter } from '@zip.js/zip.js'
import Ajv2020 from 'ajv/dist/2020.js'

import type { InferredVersionInfo } from './infer'
import schema from './schemas/sfpack/1/manifest.schema.json'

interface SfpackManifest {
	name: string
	versionId: string
	gameVersions: string[]
	mods: { projectId: string; versionId: string; sha512: string }[]
}

let validate: ReturnType<Ajv2020['compile']> | undefined

export class InvalidSfpackError extends Error {}

export async function inferSfpackVersion(entries: FileEntry[]): Promise<InferredVersionInfo> {
	if (entries.length !== 1 || entries[0]?.filename !== 'shroudforge.index.json') {
		throw new InvalidSfpackError('A .sfpack archive must contain only shroudforge.index.json.')
	}
	let value: unknown
	try {
		value = JSON.parse(await entries[0].getData(new TextWriter()))
	} catch {
		throw new InvalidSfpackError('The .sfpack manifest is not valid JSON.')
	}
	validate ??= new Ajv2020({ strict: false }).compile<SfpackManifest>(schema)
	if (!validate(value))
		throw new InvalidSfpackError('The .sfpack manifest does not match schema 1.')
	const pack = value as SfpackManifest
	if (new Set(pack.mods.map((mod) => mod.projectId)).size !== pack.mods.length) {
		throw new InvalidSfpackError('The .sfpack manifest contains duplicate projects.')
	}
	return {
		package_fields: ['name', 'version_number', 'game_versions', 'loaders'],
		name: pack.name,
		version_number: pack.versionId,
		game_versions: pack.gameVersions,
		loaders: ['sfpack'],
	}
}
