import { parse, TYPE } from '@formatjs/icu-messageformat-parser'
import { existsSync } from 'node:fs'
import { readFile, readdir, writeFile } from 'node:fs/promises'
import { dirname, join, relative, resolve } from 'node:path'
import { fileURLToPath, pathToFileURL } from 'node:url'

type MessageFile = Record<string, string>
type ArgUse = 'argument' | 'number' | 'date' | 'time' | 'plural' | 'select'
type Contract = { args: Record<string, ArgUse[]>; tags: string[]; selectBranches: Record<string, string[]> }
type Issue = { file: string; key: string; reason: string }

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), '..')
const DEFAULT_LOCALE = 'en-US'
const TRANSLATION_COLLECTIONS = [
	{
		scope: 'apps/frontend',
		source: 'apps/frontend/src/locales/en-US/index.json',
		locales: 'apps/frontend/src/locales',
	},
	{
		scope: 'packages/ui',
		source: 'packages/ui/src/locales/en-US/index.json',
		locales: 'packages/ui/src/locales',
	},
	{
		scope: 'packages/moderation',
		source: 'packages/moderation/src/locales/en-US/index.json',
		locales: 'packages/moderation/src/locales',
	},
]

function stable<T extends string>(items: Set<T>) {
	return [...items].sort()
}

function stableRecord<T extends string>(items: Map<string, Set<T>>) {
	return Object.fromEntries(
		[...items.entries()]
			.sort(([a], [b]) => a.localeCompare(b))
			.map(([key, values]) => [key, stable(values)]),
	)
}

export function contractFromMessage(message: string, label: string): Contract {
	const args = new Map<string, Set<ArgUse>>()
	const tags = new Set<string>()
	const selectBranches = new Map<string, Set<string>>()

	function addArg(name: string, use: ArgUse) {
		const uses = args.get(name) ?? new Set<ArgUse>()
		uses.add(use)
		args.set(name, uses)
	}

	function addSelectBranch(name: string, selector: string) {
		const branches = selectBranches.get(name) ?? new Set<string>()
		branches.add(selector)
		selectBranches.set(name, branches)
	}

	function visit(elements: ReturnType<typeof parse>) {
		for (const element of elements) {
			switch (element.type) {
				case TYPE.argument:
					addArg(element.value, 'argument')
					break
				case TYPE.number:
					addArg(element.value, 'number')
					break
				case TYPE.date:
					addArg(element.value, 'date')
					break
				case TYPE.time:
					addArg(element.value, 'time')
					break
				case TYPE.select: {
					addArg(element.value, 'select')
					for (const [selector, option] of Object.entries(element.options)) {
						addSelectBranch(element.value, selector)
						visit(option.value)
					}
					break
				}
				case TYPE.plural: {
					addArg(element.value, 'plural')
					for (const [selector, option] of Object.entries(element.options)) {
						visit(option.value)
					}
					break
				}
				case TYPE.tag:
					tags.add(element.value)
					visit(element.children)
					break
			}
		}
	}

	try {
		visit(parse(message, { ignoreTag: false }))
	} catch (error) {
		try {
			visit(parse(message, { ignoreTag: true }))
		} catch {
			throw new Error(`${label}: invalid ICU: ${(error as Error).message}`)
		}
	}

	return { args: stableRecord(args), tags: stable(tags), selectBranches: stableRecord(selectBranches) }
}

export function contractsEqual(a: Contract, b: Contract) {
	return JSON.stringify(a) === JSON.stringify(b)
}

export function translationCompatibleWithSource(source: Contract, translation: Contract) {
	const sourceArgs = new Map(
		Object.entries(source.args).map(([key, value]) => [key, new Set<ArgUse>(value)]),
	)
	const sourceTags = new Set(source.tags)
	const sourceSelectBranches = new Map(
		Object.entries(source.selectBranches).map(([key, value]) => [key, new Set(value)]),
	)

	for (const tag of translation.tags) {
		if (!sourceTags.has(tag)) return false
	}

	for (const tag of source.tags) {
		if (!translation.tags.includes(tag)) return false
	}

	for (const [arg, uses] of Object.entries(translation.args)) {
		const allowedUses = sourceArgs.get(arg)
		if (!allowedUses) return false

		for (const use of uses) {
			if (use === 'argument') continue

			if (use === 'number' || use === 'plural') {
				if (!allowedUses.has('number') && !allowedUses.has('plural')) return false
				continue
			}

			if (!allowedUses.has(use)) return false
		}
	}

	for (const [arg, branches] of Object.entries(translation.selectBranches)) {
		const allowedBranches = sourceSelectBranches.get(arg)
		if (!allowedBranches) return false

		for (const branch of branches) {
			if (branch !== 'other' && !allowedBranches.has(branch)) return false
		}
	}

	return true
}

async function readJson(file: string): Promise<MessageFile> {
	return JSON.parse(await readFile(file, 'utf8')) as MessageFile
}

async function writeJson(file: string, value: MessageFile) {
	await writeFile(file, `${JSON.stringify(value, null, 2)}\n`)
}

function sourceContracts(sourceFile: string, sourceMessages: MessageFile) {
	const contracts = new Map<string, Contract>()
	for (const [key, message] of Object.entries(sourceMessages)) {
		contracts.set(key, contractFromMessage(message, `${sourceFile}:${key}`))
	}
	return contracts
}

export async function pruneLocalTranslations(options: { check: boolean; scope?: string }) {
	const issues: Issue[] = []
	const collections = TRANSLATION_COLLECTIONS.filter(
		(collection) => !options.scope || collection.scope === options.scope.replace(/\/$/, ''),
	)

	for (const collection of collections) {
		const sourceFile = resolve(ROOT, collection.source)
		const source = await readJson(sourceFile)
		const contracts = sourceContracts(sourceFile, source)
		const localesRoot = resolve(ROOT, collection.locales)
		const localeDirectories = await readdir(localesRoot, { withFileTypes: true })

		for (const localeDirectory of localeDirectories) {
			if (!localeDirectory.isDirectory() || localeDirectory.name === DEFAULT_LOCALE) continue

			const translationFile = join(localesRoot, localeDirectory.name, 'index.json')
			if (!existsSync(translationFile)) continue

			const translations = await readJson(translationFile)
			let changed = false

			for (const [key, translationText] of Object.entries(translations)) {
				const sourceContract = contracts.get(key)

				if (!sourceContract) {
					delete translations[key]
					changed = true
					issues.push({ file: translationFile, key, reason: 'source key no longer exists' })
					continue
				}

				try {
					const translationContract = contractFromMessage(translationText, `${translationFile}:${key}`)
					if (!translationCompatibleWithSource(sourceContract, translationContract)) {
						delete translations[key]
						changed = true
						issues.push({
							file: translationFile,
							key,
							reason: 'translation uses unsupported ICU variables, tags, or select branches',
						})
					}
				} catch {
					delete translations[key]
					changed = true
					issues.push({ file: translationFile, key, reason: 'translation ICU is invalid' })
				}
			}

			if (changed && !options.check) await writeJson(translationFile, translations)
		}
	}

	for (const issue of issues) {
		console.log(`${relative(ROOT, issue.file)}: ${issue.key} - ${issue.reason}`)
	}

	if (options.check && issues.length > 0) {
		throw new Error(`${issues.length} stale i18n translation(s) need pruning`)
	}
}

function readOptions(args: string[]) {
	const options: Record<string, string | boolean> = {}
	for (let i = 0; i < args.length; i++) {
		const arg = args[i]
		if (!arg.startsWith('--')) continue
		const key = arg.slice(2)
		const next = args[i + 1]
		if (!next || next.startsWith('--')) {
			options[key] = true
		} else {
			options[key] = next
			i++
		}
	}
	return options
}

async function main() {
	const [command, ...rest] = process.argv.slice(2)
	const options = readOptions(rest)

	if (command === 'prune-local') {
		await pruneLocalTranslations({
			check: options.check === true,
			scope: typeof options.scope === 'string' ? options.scope : undefined,
		})
		return
	}

	throw new Error('Usage: pnpm scripts i18n-icu-contract prune-local')
}

if (import.meta.url === pathToFileURL(process.argv[1] ?? '').href) {
	main().catch((error) => {
		console.error(error)
		process.exit(1)
	})
}
