<script setup lang="ts">
import { type Labrinth, ShroudEditApiError } from '@shroudedit/api-client'
import {
	Button,
	ButtonLink,
	commonMessages,
	defineMessages,
	injectNotificationManager,
	injectShroudEditClient,
	useFormatBytes,
	useVIntl,
} from '@shroudedit/ui'
import type { Thread } from '@shroudedit/utils'
import { useQuery, useQueryClient } from '@tanstack/vue-query'

import ThreadView from '~/components/ui/thread/ThreadView.vue'
import VersionMalwareStatus from '~/components/ui/VersionMalwareStatus.vue'
import { canDownloadVersion } from '~/utils/version-malware'

type FileReview = Labrinth.TechReview.Internal.FileReport & {
	version_number: string
	versionDownloadAllowed: boolean
}

const { formatMessage } = useVIntl()
const messages = defineMessages({
	pageTitle: {
		id: 'moderation.virusTotal.project.title',
		defaultMessage: 'VirusTotal technical review - ShroudEdit',
	},
	back: { id: 'moderation.virusTotal.project.back', defaultMessage: 'Review queue' },
	projectTitle: {
		id: 'moderation.virusTotal.project.fallbackTitle',
		defaultMessage: 'Project technical review',
	},
	reviewDescription: {
		id: 'moderation.virusTotal.project.description',
		defaultMessage:
			'VirusTotal results inform this review. A safe verdict closes technical review without approving the project.',
	},
	loading: {
		id: 'moderation.virusTotal.project.loading',
		defaultMessage: 'Loading project report…',
	},
	loadError: {
		id: 'moderation.virusTotal.project.loadError',
		defaultMessage: 'Could not load the project report.',
	},
	clearanceUnavailable: {
		id: 'moderation.virusTotal.project.clearanceUnavailable',
		defaultMessage: 'The API does not provide download clearance for some files yet.',
	},
	rejectUnsafe: {
		id: 'moderation.virusTotal.project.rejectUnsafe',
		defaultMessage: 'Reject as unsafe',
	},
	filesHeading: {
		id: 'moderation.virusTotal.project.filesHeading',
		defaultMessage: 'Files and VirusTotal results',
	},
	noFiles: {
		id: 'moderation.virusTotal.project.noFiles',
		defaultMessage: 'No version files are available.',
	},
	download: { id: 'moderation.virusTotal.project.download', defaultMessage: 'Download' },
	runScan: {
		id: 'moderation.virusTotal.project.runScan',
		defaultMessage: 'Run VirusTotal scan again',
	},
	queueing: { id: 'moderation.virusTotal.project.queueing', defaultMessage: 'Queuing…' },
	version: { id: 'moderation.virusTotal.project.version', defaultMessage: 'Version {version}' },
	sha256: { id: 'moderation.virusTotal.project.sha256', defaultMessage: 'SHA-256: {hash}' },
	malicious: {
		id: 'moderation.virusTotal.project.malicious',
		defaultMessage: 'Malicious: {count}',
	},
	suspicious: {
		id: 'moderation.virusTotal.project.suspicious',
		defaultMessage: 'Suspicious: {count}',
	},
	undetected: {
		id: 'moderation.virusTotal.project.undetected',
		defaultMessage: 'Undetected: {count}',
	},
	checked: { id: 'moderation.virusTotal.project.checked', defaultMessage: 'Checked {date}' },
	notClean: {
		id: 'moderation.virusTotal.project.notClean',
		defaultMessage: 'This file is not marked clean. VirusTotal status: {status}.',
	},
	decision: {
		id: 'moderation.virusTotal.project.decision',
		defaultMessage: 'Decision: {decision}',
	},
	pending: { id: 'moderation.virusTotal.project.pending', defaultMessage: 'pending' },
	localFalsePositive: {
		id: 'moderation.virusTotal.project.localFalsePositive',
		defaultMessage: 'False positive for this file',
	},
	localUnsafe: {
		id: 'moderation.virusTotal.project.localUnsafe',
		defaultMessage: 'Mark unsafe locally',
	},
	resetLocal: {
		id: 'moderation.virusTotal.project.resetLocal',
		defaultMessage: 'Reset local decision',
	},
	threadHeading: {
		id: 'moderation.virusTotal.project.threadHeading',
		defaultMessage: 'Moderation thread',
	},
	scanQueued: {
		id: 'moderation.virusTotal.notifications.scanQueued',
		defaultMessage: 'VirusTotal scan queued',
	},
	queueScanError: {
		id: 'moderation.virusTotal.notifications.queueScanError',
		defaultMessage: 'Could not queue VirusTotal scan',
	},
	saveFindingError: {
		id: 'moderation.virusTotal.notifications.saveFindingError',
		defaultMessage: 'Could not save finding decision',
	},
	reviewCompleted: {
		id: 'moderation.virusTotal.notifications.reviewCompleted',
		defaultMessage: 'Technical review completed',
	},
	rejected: {
		id: 'moderation.virusTotal.notifications.rejected',
		defaultMessage: 'The project was rejected.',
	},
	submitError: {
		id: 'moderation.virusTotal.notifications.submitError',
		defaultMessage: 'Could not submit review',
	},
})

useHead({ title: formatMessage(messages.pageTitle) })

const route = useRoute()
const router = useRouter()
const client = injectShroudEditClient()
const queryClient = useQueryClient()
const { addNotification } = injectNotificationManager()
const formatBytes = useFormatBytes()
const projectId = computed(() => String(route.params.project))
const pendingFileActions = ref<Set<string>>(new Set())
const pendingFindingActions = ref<Set<string>>(new Set())
const isSubmitting = ref(false)

const reviewQuery = useQuery({
	queryKey: computed(() => ['virustotal-review', projectId.value]),
	queryFn: () => client.labrinth.tech_review_internal.getProjectReport(projectId.value),
	refetchInterval: (query) =>
		query.state.data?.project_report?.versions.some((version) =>
			version.files.some((file) =>
				file.malware
					? file.malware.status === 'pending' && !file.malware.failed
					: file.status === 'queued' || file.status === 'submitted',
			),
		)
			? 15_000
			: false,
})
const projectQuery = useQuery({
	queryKey: computed(() => ['project', projectId.value]),
	queryFn: () => client.labrinth.projects_v3.get(projectId.value),
})

const files = computed<FileReview[]>(
	() =>
		reviewQuery.data.value?.project_report?.versions
			.flatMap((version) =>
				version.files.map((file) => ({
					...file,
					version_number: version.version_number,
					versionDownloadAllowed: canDownloadVersion(version),
					findings: file.findings.filter(
						(finding) => finding.category === 'malicious' || finding.category === 'suspicious',
					),
				})),
			)
			?.sort(
				(a, b) =>
					Number(b.malware?.status === 'blocked' || b.malware?.failed === true) -
					Number(a.malware?.status === 'blocked' || a.malware?.failed === true),
			) ?? [],
)
const reportHttpStatus = computed(() => {
	const error = reviewQuery.error.value
	return error instanceof ShroudEditApiError ? error.statusCode : undefined
})
const thread = computed(() => reviewQuery.data.value?.thread as unknown as Thread | undefined)

function stats(file: FileReview) {
	return file.analysis?.stats ?? {}
}

function findingKey(file: FileReview, finding: Labrinth.TechReview.Internal.VirusTotalFinding) {
	return `${file.file_id}:${finding.engine_name}:${finding.category}:${finding.result}`
}

async function rerunScan(file: FileReview) {
	pendingFileActions.value.add(file.file_id)
	try {
		await client.labrinth.tech_review_internal.runFileScan(file.file_id)
		addNotification({
			type: 'success',
			title: formatMessage(messages.scanQueued),
			text: file.file_name,
		})
		await Promise.all([
			queryClient.invalidateQueries({ queryKey: ['virustotal-review', projectId.value] }),
			queryClient.invalidateQueries({ queryKey: ['virustotal-review-queue'] }),
			queryClient.invalidateQueries({ queryKey: ['project', projectId.value, 'versions'] }),
		])
	} catch (error) {
		addNotification({
			type: 'error',
			title: formatMessage(messages.queueScanError),
			text: String(error),
		})
	} finally {
		pendingFileActions.value.delete(file.file_id)
	}
}

async function setFindingVerdict(
	file: FileReview,
	finding: Labrinth.TechReview.Internal.VirusTotalFinding,
	verdict: Labrinth.TechReview.Internal.VirusTotalVerdict | null,
) {
	if (!file.sha256 || !file.analysis_id) return
	const key = findingKey(file, finding)
	pendingFindingActions.value.add(key)
	try {
		await client.labrinth.tech_review_internal.updateFindingVerdicts([
			{
				file_id: file.file_id,
				analysis_id: file.analysis_id,
				sha256: file.sha256,
				engine_name: finding.engine_name,
				category: finding.category,
				result: finding.result,
				scope: 'local',
				verdict,
			},
		])
		await Promise.all([
			queryClient.invalidateQueries({ queryKey: ['virustotal-review', projectId.value] }),
			queryClient.invalidateQueries({ queryKey: ['virustotal-review-queue'] }),
			queryClient.invalidateQueries({ queryKey: ['project', projectId.value, 'versions'] }),
		])
	} catch (error) {
		addNotification({
			type: 'error',
			title: formatMessage(messages.saveFindingError),
			text: String(error),
		})
	} finally {
		pendingFindingActions.value.delete(key)
	}
}

async function rejectProject() {
	isSubmitting.value = true
	try {
		await client.labrinth.tech_review_internal.submitProject(projectId.value, { verdict: 'unsafe' })
		addNotification({
			type: 'success',
			title: formatMessage(messages.reviewCompleted),
			text: formatMessage(messages.rejected),
		})
		await router.push('/moderation/technical-review')
	} catch (error) {
		addNotification({
			type: 'error',
			title: formatMessage(messages.submitError),
			text: String(error),
		})
	} finally {
		isSubmitting.value = false
	}
}
</script>

<template>
	<div class="flex flex-col gap-5">
		<div class="flex flex-wrap items-start justify-between gap-3">
			<div>
				<ButtonLink to="/moderation/technical-review" type="quiet" class="mb-2">
					← {{ formatMessage(messages.back) }}
				</ButtonLink>
				<h2 class="m-0 text-2xl font-bold">
					{{ projectQuery.data.value?.name ?? formatMessage(messages.projectTitle) }}
				</h2>
				<p class="mb-0 mt-1 text-secondary">
					{{ formatMessage(messages.reviewDescription) }}
				</p>
			</div>
			<div v-if="reviewQuery.data.value?.needs_review" class="flex gap-2">
				<Button
					type="colored"
					color="red"
					:disabled="isSubmitting || !reviewQuery.data.value?.project_report"
					@click="rejectProject"
				>
					{{ formatMessage(messages.rejectUnsafe) }}
				</Button>
			</div>
		</div>

		<div v-if="reviewQuery.isLoading.value" class="rounded-xl bg-surface-2 p-5 text-secondary">
			{{ formatMessage(messages.loading) }}
		</div>
		<div
			v-else-if="reviewQuery.error.value"
			class="flex flex-wrap items-center gap-3 rounded-xl bg-surface-2 p-5"
		>
			<span class="text-red">
				{{ formatMessage(messages.loadError) }}
				<span v-if="reportHttpStatus">(HTTP {{ reportHttpStatus }})</span>
			</span>
			<Button
				type="outlined"
				:disabled="reviewQuery.isFetching.value"
				@click="reviewQuery.refetch()"
			>
				{{ formatMessage(commonMessages.retryButton) }}
			</Button>
		</div>
		<template v-else>
			<section class="rounded-xl bg-surface-2 p-4">
				<h3 class="mb-3 mt-0 text-lg font-semibold">{{ formatMessage(messages.filesHeading) }}</h3>
				<p v-if="files.some((file) => !file.malware)" class="mb-3 mt-0 text-secondary">
					{{ formatMessage(messages.clearanceUnavailable) }}
				</p>
				<div v-if="files.length === 0" class="text-secondary">
					{{ formatMessage(messages.noFiles) }}
				</div>
				<div v-else class="flex flex-col gap-3">
					<article v-for="file in files" :key="file.file_id" class="rounded-xl bg-surface-1 p-4">
						<div class="flex flex-wrap items-start justify-between gap-3">
							<div class="min-w-0">
								<h4 class="m-0 break-all font-semibold">{{ file.file_name }}</h4>
								<p class="mb-0 mt-1 text-sm text-secondary">
									{{ formatMessage(messages.version, { version: file.version_number }) }} ·
									{{ formatBytes(file.file_size) }} · {{ file.status }}
								</p>
								<p v-if="file.sha256" class="mb-0 mt-1 break-all font-mono text-xs text-secondary">
									{{ formatMessage(messages.sha256, { hash: file.sha256 }) }}
								</p>
								<p v-if="file.error" class="mb-0 mt-2 text-sm text-red">{{ file.error }}</p>
							</div>
							<div class="flex flex-wrap gap-2">
								<VersionMalwareStatus
									:scan="
										file.malware ?? {
											status: 'pending',
											analysis_url: file.analysis_url,
											failed: file.status === 'failed' || file.status === 'unknown',
										}
									"
								/>
								<ButtonLink
									v-if="file.versionDownloadAllowed"
									:href="file.download_url"
									:color="file.malware?.status === 'manually_approved' ? 'blue' : 'green'"
									target="_blank"
									type="outlined"
									>{{ formatMessage(messages.download) }}</ButtonLink
								>
								<Button :disabled="pendingFileActions.has(file.file_id)" @click="rerunScan(file)">
									{{
										formatMessage(
											pendingFileActions.has(file.file_id) ? messages.queueing : messages.runScan,
										)
									}}
								</Button>
							</div>
						</div>

						<div v-if="file.analysis" class="mt-3 flex flex-wrap gap-3 text-sm">
							<span>{{
								formatMessage(messages.malicious, { count: stats(file).malicious ?? 0 })
							}}</span>
							<span>{{
								formatMessage(messages.suspicious, { count: stats(file).suspicious ?? 0 })
							}}</span>
							<span>{{
								formatMessage(messages.undetected, { count: stats(file).undetected ?? 0 })
							}}</span>
							<span v-if="file.checked_at" class="text-secondary">{{
								formatMessage(messages.checked, {
									date: new Date(file.checked_at).toLocaleString(),
								})
							}}</span>
						</div>

						<p v-if="file.status !== 'completed'" class="mb-0 mt-3 text-sm text-secondary">
							{{ formatMessage(messages.notClean, { status: file.status }) }}
						</p>

						<div v-if="file.findings.length > 0" class="mt-4 flex flex-col gap-2">
							<div
								v-for="finding in file.findings"
								:key="findingKey(file, finding)"
								class="rounded-lg border border-surface-5 p-3"
							>
								<div class="flex flex-wrap items-center justify-between gap-2">
									<div>
										<strong>{{ finding.engine_name }}</strong>
										<span class="ml-2 text-sm text-secondary">{{ finding.category }}</span>
										<span v-if="finding.result" class="ml-2 text-sm">{{ finding.result }}</span>
									</div>
									<span class="text-sm text-secondary">
										{{
											formatMessage(messages.decision, {
												decision: finding.local_verdict ?? formatMessage(messages.pending),
											})
										}}
									</span>
								</div>
								<div
									v-if="
										file.status === 'completed' &&
										file.malware?.analysis_url &&
										file.sha256 &&
										['malicious', 'suspicious'].includes(finding.category)
									"
									class="mt-3 flex flex-wrap gap-2"
								>
									<Button
										type="outlined"
										:disabled="pendingFindingActions.has(findingKey(file, finding))"
										@click="setFindingVerdict(file, finding, 'safe')"
										>{{ formatMessage(messages.localFalsePositive) }}</Button
									>
									<Button
										type="outlined"
										:disabled="pendingFindingActions.has(findingKey(file, finding))"
										@click="setFindingVerdict(file, finding, 'unsafe')"
										>{{ formatMessage(messages.localUnsafe) }}</Button
									>
									<Button
										v-if="finding.local_verdict"
										type="quiet"
										:disabled="pendingFindingActions.has(findingKey(file, finding))"
										@click="setFindingVerdict(file, finding, null)"
										>{{ formatMessage(messages.resetLocal) }}</Button
									>
								</div>
							</div>
						</div>
					</article>
				</div>
			</section>

			<section v-if="thread" class="rounded-xl bg-surface-2 p-4">
				<h3 class="mb-3 mt-0 text-lg font-semibold">{{ formatMessage(messages.threadHeading) }}</h3>
				<!-- @vue-expect-error ThreadView still consumes the legacy thread utility type -->
				<ThreadView :thread="thread" primary-action="note" />
			</section>
		</template>
	</div>
</template>
