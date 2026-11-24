import { type FileEntry, TextWriter } from '@zip.js/zip.js'
import Ajv2020 from 'ajv/dist/2020.js'

import type { InferredVersionInfo } from './infer'
import extensionSchema from './schemas/shroudforge-1.8.1/extended.mod.schema.json'
import manifestSchema from './schemas/shroudforge-1.8.1/manifest.schema.json'
import localeSchema from './schemas/shroudforge-1.8.1/mod-locale.schema.json'

const ajv = new Ajv2020({ ownProperties: true, strict: false })
interface ModManifest {
	id: string
	name: string
	version: string
}

const validateManifest = ajv.compile<ModManifest>({
	type: manifestSchema.type,
	required: manifestSchema.required,
	properties: Object.fromEntries(
		Object.entries(manifestSchema.properties).filter(([key]) =>
			manifestSchema.required.includes(key),
		),
	),
})
const validateLauncher = ajv.compile<'EML' | 'SF'>(extensionSchema.properties.launcher)
const validateTargets = ajv.compile<('client' | 'server')[]>(extensionSchema.properties.targets)
const validateChangelog = ajv.compile<string[]>(extensionSchema.properties.changelog)
const validateLocale = ajv.compile(localeSchema)

export class InvalidModManifestError extends Error {
	constructor(readonly filename?: string) {
		super(filename)
	}
}

async function readJson(entry?: FileEntry): Promise<Record<string, unknown> | undefined> {
	if (!entry) return undefined
	try {
		const data: unknown = JSON.parse(await entry.getData(new TextWriter()))
		return data && typeof data === 'object' && !Array.isArray(data)
			? (data as Record<string, unknown>)
			: undefined
	} catch {
		return undefined
	}
}

export async function inferShroudForgeVersion(
	entries: FileEntry[],
	requireManifest: boolean,
): Promise<InferredVersionInfo> {
	const packagePaths = new Set<string>()
	const manifests = new Map<string, ModManifest>()
	for (const entry of entries) {
		const match = /^((?:[^/]+\/)?(?:mods\/[^/]+\/)?)(mod|extended\.mod)\.json$/.exec(entry.filename)
		if (!match) continue

		const [, packagePath, manifestKind] = match
		packagePaths.add(packagePath)
		if (manifestKind === 'mod') {
			const manifest = await readJson(entry)
			if (!validateManifest(manifest)) {
				throw new InvalidModManifestError(entry.filename)
			}
			manifests.set(packagePath, manifest)
		}
	}
	if (requireManifest && manifests.size === 0) throw new InvalidModManifestError()

	const packagePath = packagePaths.has('') ? '' : [...packagePaths][0]
	if (packagePath === undefined || (packagePath !== '' && packagePaths.size > 1)) return {}

	const manifest = manifests.get(packagePath)
	const extensionFile = entries.find(
		({ filename }) => filename === `${packagePath}extended.mod.json`,
	)
	const extension = await readJson(extensionFile)

	const localePath = `${packagePath}locales/`
	for (const entry of entries) {
		if (
			entry.filename.startsWith(localePath) &&
			/^[^/]+\.json$/.test(entry.filename.slice(localePath.length))
		) {
			const locale = await readJson(entry)
			if (locale && !validateLocale(locale)) {
				console.debug('ShroudForge locale does not match its schema:', entry.filename)
			}
		}
	}

	let loader: 'eml' | 'shroudforge' | undefined
	if (extension) {
		const launcher = extension.launcher
		if (launcher === undefined || validateLauncher(launcher)) {
			loader = launcher === 'EML' ? 'eml' : 'shroudforge'
		}
	} else if (manifest && !extensionFile) {
		loader = 'eml'
	}

	const targets = extension?.targets
	let environment: InferredVersionInfo['environment']
	if (validateTargets(targets)) {
		environment =
			targets.length === 2
				? 'client_or_server'
				: targets[0] === 'server'
					? 'server_only'
					: 'client_only'
	} else if (targets === undefined) {
		if (loader === 'eml') environment = 'client_or_server'
		if (loader === 'shroudforge') environment = 'client_only'
	}

	const changelog = extension?.changelog
	return {
		name: manifest?.name,
		version_number: manifest?.version,
		loaders: loader ? [loader] : undefined,
		environment,
		changelog: validateChangelog(changelog) ? changelog.join('\n\n') : undefined,
	}
}
