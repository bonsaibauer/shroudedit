import { defineMessages } from '@shroudedit/ui'

const projectTypeMessages = defineMessages({
	mod: {
		id: 'project-type.mod.singular',
		defaultMessage: 'Mod',
	},
	mods: {
		id: 'project-type.mod.plural',
		defaultMessage: 'Mods',
	},
	modpack: {
		id: 'project-type.modpack.singular',
		defaultMessage: 'Modpack',
	},
	modpacks: {
		id: 'project-type.modpack.plural',
		defaultMessage: 'Modpacks',
	},
	schematic: {
		id: 'project-type.schematic.singular',
		defaultMessage: 'Schematic',
	},
	schematics: {
		id: 'project-type.schematic.plural',
		defaultMessage: 'Schematics',
	},
	server: {
		id: 'project-type.server.singular',
		defaultMessage: 'Server',
	},
	servers: {
		id: 'project-type.server.plural',
		defaultMessage: 'Servers',
	},
	project: {
		id: 'project-type.project.singular',
		defaultMessage: 'Project',
	},
	projects: {
		id: 'project-type.project.plural',
		defaultMessage: 'Projects',
	},
	collection: {
		id: 'project-type.collection.singular',
		defaultMessage: 'Collection',
	},
	collections: {
		id: 'project-type.collection.plural',
		defaultMessage: 'Collections',
	},
})

type ExtractSingulars<K extends string> = K extends `${infer T}s` ? T : never

type ProjectType = ExtractSingulars<keyof typeof projectTypeMessages>

export function getProjectTypeMessage(type: ProjectType, plural = false) {
	return (
		projectTypeMessages[`${type}${plural ? 's' : ''}`] ??
		projectTypeMessages[`project${plural ? 's' : ''}`]
	)
}
