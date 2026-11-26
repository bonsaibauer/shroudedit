import { VersionIcon } from '@shroudedit/assets'
import { injectProjectPageContext } from '@shroudedit/ui'
import { computed } from 'vue'

import { dropdown, group, option, stage, text, toggle } from '../../types/node'

export default function () {
	const { projectV3: project } = injectProjectPageContext()

	return stage('versions', 'Versions')
		.hint("Are this project's files correct?")
		.guidance(
			'https://www.notion.so/2e15ee711bf080e4a41df61bbab49892#2e25ee711bf0804bad38e9055951ff31',
		)
		.icon(VersionIcon)
		.navigate('/versions')
		.shown(computed(() => !project.value?.enshrouded_server))
		.children(
			group().children(
				toggle('incorrect-additional-files', 'Incorrect additional files')
					.suggestedStatus('flagged')
					.message(),

				toggle('incorrect-project-type', 'Incorrect Project Type')
					.suggestedStatus('rejected')
					.children(
						dropdown('type')
							.title('Correct Project Type')
							.required()
							.none('Unknown')
							.options(
								option('mod', 'Mod').message(),
								option('modpack', 'Modpack').message(),
								option('schematic', 'Schematic').message(),
								option('server', 'Server').message(),
							),
					)
					.collect(),

				toggle('alternate-versions', 'Alternate Versions')
					.suggestedStatus('rejected')
					.children(
						dropdown('distribution')
							.title('Distribution Type')
							.required()
							.none('Unknown')
							.options(
								option('primary', 'Primary Files').message(),
								option('additional', 'Additional Files').message(),
								option('server', 'Server Files (Primary Files)')
									.shown(computed(() => project.value.project_types.includes('modpack')))
									.message(),
								option('server-additional', 'Server Files (Additional Files)')
									.shown(computed(() => project.value.project_types.includes('modpack')))
									.message(),
				option('sfpack', '.sfpack')
									.shown(computed(() => project.value.project_types.includes('modpack')))
									.message(),
							),
					)
					.collect(),

				toggle('redist-libs', 'Packed Libs')
					.shown(computed(() => project.value.project_types.includes('mod')))
					.suggestedStatus('rejected')
					.message(),

				toggle('duplicate-primary-files', 'Duplicate Primary Files')
					.suggestedStatus('flagged')
					.message(),

				toggle('unsupported', 'Unsupported')
					.suggestedStatus('rejected')
					.message((state) => ({
						INVALID_TYPE: state['invalid-type'],
					}))
					.children(text('invalid-type').title('Unsupported Type').required()),
			),
		)
}
