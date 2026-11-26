import { type FileEntry, TextWriter } from '@zip.js/zip.js'
import Ajv2020 from 'ajv/dist/2020.js'

import type { InferredVersionInfo } from './infer'
import extensionSchema from './schemas/shroudforge-1.8.2/extended.mod.schema.json'
import manifestSchema from './schemas/shroudforge-1.8.2/manifest.schema.json'
import localeSchema from './schemas/shroudforge-1.8.2/mod-locale.schema.json'

interface ModManifest {
	id: string
	name: string
	version: string
}

let validators: ReturnType<typeof createValidators> | undefined

function createValidators() {
	const ajv = new Ajv2020({ ownProperties: true, strict: false })
	return {
		validateManifest: ajv.compile<ModManifest>(manifestSchema),
		validateExtension: ajv.compile(extensionSchema),
		validateLauncher: ajv.compile<'EML' | 'SF'>(extensionSchema.properties.launcher),
		validateTargets: ajv.compile<string[]>(extensionSchema.properties.targets),
		validateReleaseChannel: ajv.compile<'release' | 'beta' | 'alpha'>(
			extensionSchema.properties.releaseChannel,
		),
		validateGameVersions: ajv.compile<string[]>(extensionSchema.properties.gameVersions),
		validateChangelog: ajv.compile<string[]>(extensionSchema.properties.changelog),
		validateLocale: ajv.compile(localeSchema),
	}
}

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

export async function inferShroudForgeVersion(entries: FileEntry[]): Promise<InferredVersionInfo> {
	const {
		validateManifest,
		validateExtension,
		validateLauncher,
		validateTargets,
		validateReleaseChannel,
		validateGameVersions,
		validateChangelog,
		validateLocale,
	} = (validators ??= createValidators())
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
	const packagePath = packagePaths.has('') ? '' : [...packagePaths][0]
	if (packagePath === undefined || (packagePath !== '' && packagePaths.size > 1)) return {}

	const manifest = manifests.get(packagePath)
	const extensionFile = entries.find(
		({ filename }) => filename === `${packagePath}extended.mod.json`,
	)
	const extension = await readJson(extensionFile)
	if (extensionFile && !validateExtension(extension)) {
		throw new InvalidModManifestError(extensionFile.filename)
	}

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
		loader = entries.some(
			({ filename }) =>
				filename === `${packagePath}src/mod.lua` || filename === `${packagePath}native-plugin.ini`,
		)
			? 'shroudforge'
			: 'eml'
	}

	const targets = extension?.targets
	let environment: InferredVersionInfo['environment']
	if (validateTargets(targets)) {
		if (targets.length === 1 && targets[0] === 'client') environment = 'client_only'
		if (targets.length === 1 && targets[0]?.includes('_')) {
			environment = targets[0] as InferredVersionInfo['environment']
		}
	} else if (targets === undefined) {
		if (loader === 'shroudforge') environment = 'client_only'
	}

	const changelog = extension?.changelog
	const releaseChannel = extension?.releaseChannel
	const gameVersions = extension?.gameVersions
	const packageFields = manifest ? ['name', 'version_number'] : []
	if (validateReleaseChannel(releaseChannel)) packageFields.push('version_type')
	if (validateGameVersions(gameVersions)) packageFields.push('game_versions')
	if (validateTargets(targets)) packageFields.push('environment')
	if (validateChangelog(changelog)) packageFields.push('changelog')
	if (loader) packageFields.push('loaders')
	return {
		mod_id: manifest?.id,
		package_fields: packageFields,
		name: manifest?.name,
		version_number: manifest?.version,
		loaders: loader ? [loader] : undefined,
		version_type: validateReleaseChannel(releaseChannel) ? releaseChannel : undefined,
		game_versions: validateGameVersions(gameVersions) ? gameVersions : undefined,
		environment,
		changelog: validateChangelog(changelog) ? changelog.join('\n\n') : undefined,
	}
}
