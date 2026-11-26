import { defineMessages } from '@shroudedit/ui'

import type { NagDefinitions } from './types.ts'

const messages = defineMessages({
	featureTitle: {
		id: 'nags.feature-gallery-image.title',
		defaultMessage: 'Feature a gallery image',
	},
	feature: {
		id: 'nags.feature-gallery-image.description',
		defaultMessage:
			'The featured gallery image is often how your project makes its first impression.',
	},
	uploadTitle: { id: 'nags.upload-gallery-image.title', defaultMessage: 'Upload a gallery image' },
	upload: {
		id: 'nags.upload-gallery-image.description',
		defaultMessage:
			'At least one gallery image is required to showcase the content of your {type}.',
	},
})

export const galleryNags = {
	'feature-gallery-image': {
		title: messages.featureTitle,
		description: messages.feature,
		destination: 'gallery',
	},
	'upload-gallery-image': {
		title: messages.uploadTitle,
		description: messages.upload,
		destination: 'gallery',
	},
} satisfies NagDefinitions
