import { type Entry, type FileEntry, TextWriter } from '@zip.js/zip.js'
import Ajv2020 from 'ajv/dist/2020.js'

import type { InferredVersionInfo } from './infer'
import extendedSchema from './schemas/shroudforge-1.8.2/extended.mod.schema.json'
import modSchema from './schemas/shroudforge-1.8.2/manifest.schema.json'
import schema from './schemas/shroudforge-1.8.2/modpack.schema.json'

interface SfpackManifest {
	id: string
	name: string
	version: string
	gameVersions: string[]
	mods: (
		| { path: string; optional?: boolean }
		| { id: string; version: string; optional?: boolean }
		| { id: string; version: string; url: string; sha512: string; optional?: boolean }
	)[]
}

let validate: ReturnType<Ajv2020['compile']> | undefined
let validateMod: ReturnType<Ajv2020['compile']> | undefined
let validateExtended: ReturnType<Ajv2020['compile']> | undefined

export class InvalidSfpackError extends Error {}

function hasControlCharacters(value: string): boolean {
	return [...value].some((character) => {
		const codePoint = character.codePointAt(0)!
		return codePoint <= 0x1f || (codePoint >= 0x7f && codePoint <= 0x9f)
	})
}

function isBundled(
	mod: SfpackManifest['mods'][number],
): mod is { path: string; optional?: boolean } {
	return 'path' in mod
}

export async function inferSfpackVersion(entries: Entry[]): Promise<InferredVersionInfo> {
	const manifestEntries = entries.filter(
		(entry) => entry.filename === 'modpack.json' && !entry.directory,
	)
	if (manifestEntries.length !== 1) {
		throw new InvalidSfpackError('A .sfpack archive needs one root modpack.json.')
	}
	const manifest = manifestEntries[0] as FileEntry
	if (manifest.uncompressedSize > 1024 * 1024) {
		throw new InvalidSfpackError('modpack.json exceeds 1 MiB.')
	}
	let value: unknown
	try {
		value = JSON.parse(await manifest.getData(new TextWriter()))
	} catch {
		throw new InvalidSfpackError('modpack.json is not valid JSON.')
	}
	validate ??= new Ajv2020({ strict: false }).compile<SfpackManifest>(schema)
	if (!validate(value)) {
		throw new InvalidSfpackError('modpack.json does not match the Modloader 1.8.2 schema.')
	}
	const pack = value as SfpackManifest
	const roots = pack.mods.filter(isBundled).map((mod) => mod.path)
	const ids = pack.mods.map((mod) => ('path' in mod ? mod.path.split('/').at(-1)! : mod.id))
	if (new Set(ids.map((id) => id.toLowerCase())).size !== ids.length) {
		throw new InvalidSfpackError('modpack.json contains duplicate mod IDs.')
	}
	for (const mod of pack.mods) {
		if (!('url' in mod)) continue
		let url: URL
		try {
			url = new URL(mod.url)
		} catch {
			throw new InvalidSfpackError(`External mod ${mod.id} has an invalid URL.`)
		}
		if (url.protocol !== 'https:' || !url.hostname || url.username || url.password) {
			throw new InvalidSfpackError(`External mod ${mod.id} needs an HTTPS URL without credentials.`)
		}
	}
	const names = new Set<string>()
	const populated = new Set<string>()
	let expanded = 0
	if (entries.length > 10000) {
		throw new InvalidSfpackError('The .sfpack archive contains too many entries.')
	}
	for (const entry of entries) {
		const name = entry.filename
		const path = name.replace(/\/$/, '')
		const owner = roots.find((root) => path === root || path.startsWith(`${root}/`))
		if (
			name.startsWith('/') ||
			name.length > 4096 ||
			name.includes('\\') ||
			hasControlCharacters(name) ||
			/[:<>"?*|]/.test(name) ||
			path
				.split('/')
				.some(
					(part) =>
						!part ||
						part === '.' ||
						part === '..' ||
						/[ .]$/.test(part) ||
						/^(?:con|prn|aux|nul|com[1-9]|lpt[1-9]|conin\$|conout\$)(?:\.|$)/i.test(part),
				) ||
			names.has(path.toLowerCase()) ||
			entry.encrypted ||
			![0, 8].includes(entry.compressionMethod) ||
			(entry.unixMode !== undefined && (entry.unixMode & 0o170000) === 0o120000) ||
			(owner === path && !entry.directory) ||
			(name !== 'modpack.json' && !owner && !(entry.directory && path === 'mods'))
		) {
			throw new InvalidSfpackError(`Invalid or undeclared .sfpack entry: ${name}`)
		}
		names.add(path.toLowerCase())
		if (!entry.directory) {
			expanded += (entry as FileEntry).uncompressedSize
			if (expanded > 512 * 1024 * 1024) {
				throw new InvalidSfpackError('The .sfpack contents exceed 512 MiB.')
			}
			if (owner) populated.add(owner)
		}
	}
	if (roots.some((root) => !populated.has(root))) {
		throw new InvalidSfpackError('A bundled mod folder is empty or missing.')
	}
	for (const root of roots) {
		for (const [name, isMod] of [
			['mod.json', true],
			['extended.mod.json', false],
		] as const) {
			const path = `${root}/${name}`
			const entry = entries.find(
				(candidate) => candidate.filename === path && !candidate.directory,
			) as FileEntry | undefined
			if (!entry) continue
			if (entry.uncompressedSize > 1024 * 1024) {
				throw new InvalidSfpackError(`${path} exceeds 1 MiB.`)
			}
			let contents: unknown
			try {
				contents = JSON.parse(await entry.getData(new TextWriter()))
			} catch {
				throw new InvalidSfpackError(`${path} is not valid JSON.`)
			}
			if (isMod) {
				validateMod ??= new Ajv2020({ strict: false }).compile(modSchema)
				if (!validateMod(contents) || (contents as { id: string }).id !== root.split('/').at(-1)) {
					throw new InvalidSfpackError(`${path} has an invalid schema or ID.`)
				}
			} else {
				validateExtended ??= new Ajv2020({ strict: false }).compile(extendedSchema)
				if (!validateExtended(contents)) {
					throw new InvalidSfpackError(`${path} does not match the Modloader 1.8.2 schema.`)
				}
			}
		}
	}
	return {
		package_fields: ['name', 'version_number', 'game_versions', 'loaders'],
		modpack_contents: pack.mods,
		mod_id: pack.id,
		name: pack.name,
		version_number: pack.version,
		game_versions: pack.gameVersions,
		loaders: ['sfpack'],
	}
}
