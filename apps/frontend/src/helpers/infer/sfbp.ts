import Ajv2020 from 'ajv/dist/2020.js'
import schema from './schemas/sfbp/8/manifest.schema.json'
import type { InferredVersionInfo } from './infer'

interface Blueprint {
	formatVersion: 8
	extent: [number, number, number]
	voxels?: { dimensions: [number, number, number]; cells: number[]; coverage: number[] }
	props: { rotation: number[]; templateUuidHighHex: string; templateUuidLowHex: string }[]
}
const validate = new Ajv2020({ strict: false, ownProperties: true }).compile<Blueprint>(schema)

export async function inferSfbpMetadata(file: File): Promise<InferredVersionInfo> {
	if (file.size > 95 * 1024 * 1024) throw new Error('SFBP uploads cannot exceed 95 MiB')
	const blueprint: unknown = JSON.parse(await file.text())
	if (!validate(blueprint)) throw new Error('New blueprint uploads require valid SFBP JSON format 8')
	if (blueprint.voxels) {
		const { dimensions, cells, coverage } = blueprint.voxels
		const count = dimensions.reduce((total, value) => total * value, 1)
		if (count > 65_536 || cells.length !== count || coverage.length !== count || coverage.some((state, index) => state === 2 && cells[index] === 0)) {
			throw new Error('SFBP voxel cells and coverage must match the dimensions')
		}
	}
	for (const prop of blueprint.props) {
		const norm = prop.rotation.reduce((total, value) => total + value * value, 0)
		if (!Number.isFinite(norm) || norm < 1e-12 || (/^0+$/.test(prop.templateUuidHighHex) && /^0+$/.test(prop.templateUuidLowHex))) {
			throw new Error('SFBP contains an invalid prop transform or template UUID')
		}
	}
	const dimensions = blueprint.voxels?.dimensions ?? blueprint.extent.map(Math.ceil)
	if (dimensions.some((value) => !Number.isFinite(value) || value < 1 || value > 65_536)) throw new Error('SFBP dimensions must be between 1 and 65536')
	return { schematic_format_version: 8, schematic_width: dimensions[0], schematic_height: dimensions[1], schematic_depth: dimensions[2] }
}
