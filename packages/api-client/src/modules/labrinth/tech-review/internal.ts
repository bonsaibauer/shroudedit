import { AbstractModule } from '../../../core/abstract-module'
import type { Labrinth } from '../types'

export class LabrinthTechReviewInternalModule extends AbstractModule {
	public getModuleID(): string {
		return 'labrinth_tech_review_internal'
	}

	public async searchProjects(
		params: Labrinth.TechReview.Internal.SearchProjectsRequest,
	): Promise<Labrinth.TechReview.Internal.SearchResponse> {
		return this.client.request<Labrinth.TechReview.Internal.SearchResponse>(
			'/moderation/tech-review/search',
			{
				api: 'labrinth',
				version: 'internal',
				method: 'POST',
				body: params,
			},
		)
	}

	public async getProjectReport(
		projectId: string,
	): Promise<Labrinth.TechReview.Internal.ProjectReportResponse> {
		return this.client.request<Labrinth.TechReview.Internal.ProjectReportResponse>(
			`/moderation/tech-review/project/${projectId}`,
			{
				api: 'labrinth',
				version: 'internal',
				method: 'GET',
			},
		)
	}

	public async updateFindingVerdicts(
		updates: Labrinth.TechReview.Internal.UpdateFindingVerdictRequest[],
	): Promise<void> {
		return this.client.request<void>('/moderation/tech-review/finding-verdicts', {
			api: 'labrinth',
			version: 'internal',
			method: 'PATCH',
			body: updates,
		})
	}

	public async submitProject(
		projectId: string,
		data: Labrinth.TechReview.Internal.SubmitProjectRequest,
	): Promise<void> {
		return this.client.request<void>(`/moderation/tech-review/submit/${projectId}`, {
			api: 'labrinth',
			version: 'internal',
			method: 'POST',
			body: data,
		})
	}

	public async runFileScan(fileId: string): Promise<void> {
		return this.client.request<void>(`/virustotal/run/${fileId}`, {
			api: 'labrinth',
			version: 'internal',
			method: 'POST',
		})
	}
}
