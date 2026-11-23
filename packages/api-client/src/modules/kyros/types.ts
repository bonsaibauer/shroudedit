export namespace Kyros {
	export namespace Content {
		export namespace v1 {
			export type KnownPropertiesFields = {
				allow_cheats?: string | null
				allow_flight?: string | null
				difficulty?: string | null
				enforce_whitelist?: string | null
				force_gamemode?: string | null
				gamemode?: string | null
				generate_structures?: string | null
				generator_settings?: string | null
				hardcore?: string | null
				level_seed?: string | null
				level_type?: string | null
				max_players?: string | null
				max_tick_time?: string | null
				motd?: string | null
				pause_when_empty_seconds?: string | null
				player_idle_timeout?: string | null
				require_resource_pack?: string | null
				resource_pack?: string | null
				resource_pack_id?: string | null
				resource_pack_sha1?: string | null
				simulation_distance?: string | null
				spawn_protection?: string | null
				sync_chunk_writes?: string | null
				view_distance?: string | null
				white_list?: string | null
			}

			export type PropertiesFields = {
				known: KnownPropertiesFields
				custom?: Record<string, string>
			}

		}
	}

	export namespace UploadSessions {
		export namespace v1 {
			export type Scope = 'content' | 'files'
			export type UploadSessionStatus =
				| 'active'
				| 'uploading'
				| 'finalizing'
				| 'cancelled'
				| 'finalized'
				| 'expired'

			export interface UploadSessionResponse {
				upload_id: string
				status: UploadSessionStatus
				created_at: number
				updated_at: number
				last_upload_at: number | null
				expires_at: number
				entry_count: number
				uploaded_byte_count: number
			}

			export interface GetUploadSessionResponse {
				session: UploadSessionResponse | null
			}
		}
	}

	export namespace Files {
		export namespace v1 {
			export interface FullWorldDownloadAuthorization {
				token: string
			}

			export type DescendantType = 'regular' | 'directory' | 'symlink' | 'other'

			export interface FileStatRequest {
				path: string
			}

			export interface FileStatResponse {
				name: string
				full_path: string
				size_bytes: number
				type: DescendantType
				mtime: string
				ctime: string
			}

			export type ZipRequest =
				| {
						target_type: 'Directory'
						path: string
				  }
				| {
						target_type: 'ManyPaths'
						parent: string
						include: string[]
						target: string
				  }

			export interface ZipProgress {
				progress: number
				done?: boolean
				error?: string
			}
		}

		export namespace v0 {
			export interface DirectoryItem {
				name: string
				type: 'file' | 'directory' | 'symlink'
				path: string
				modified: number
				created: number
				size?: number
				count?: number
				target?: string
			}

			export interface DirectoryResponse {
				items: DirectoryItem[]
				total: number
				current: number
			}

			export interface ExtractResult {
				modpack_name: string | null
				conflicting_files: string[]
			}
		}
	}
}
