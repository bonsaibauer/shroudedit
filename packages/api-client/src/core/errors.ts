import type { ApiErrorData, ShroudEditErrorResponse } from '../types/errors'
import { isShroudEditErrorResponse } from '../types/errors'

/**
 * Base error class for all ShroudEdit API errors
 */
export class ShroudEditApiError extends Error {
	/**
	 * HTTP status code (if available)
	 */
	readonly statusCode?: number

	/**
	 * Original error that was caught
	 */
	readonly originalError?: Error

	/**
	 * Response data from the API (if available)
	 */
	readonly responseData?: unknown

	/**
	 * Error context (e.g., module name, operation being performed)
	 */
	readonly context?: string

	constructor(message: string, data?: ApiErrorData) {
		super(message)
		this.name = 'ShroudEditApiError'

		this.statusCode = data?.statusCode
		this.originalError = data?.originalError
		this.responseData = data?.responseData
		this.context = data?.context

		// Maintains proper stack trace for where our error was thrown (only available on V8)
		if (Error.captureStackTrace) {
			Error.captureStackTrace(this, ShroudEditApiError)
		}
	}

	/**
	 * Create a ShroudEditApiError from an unknown error
	 */
	static fromUnknown(error: unknown, context?: string): ShroudEditApiError {
		if (error instanceof ShroudEditApiError) {
			return error
		}

		if (error instanceof Error) {
			return new ShroudEditApiError(error.message, {
				originalError: error,
				context,
			})
		}

		return new ShroudEditApiError(String(error), { context })
	}
}

/**
 * Error class for ShroudEdit server errors (kyros/archon)
 * Extends ShroudEditApiError with V1 error response parsing
 */
export class ShroudEditServerError extends ShroudEditApiError {
	/**
	 * V1 error information (if available)
	 */
	readonly v1Error?: ShroudEditErrorResponse

	constructor(message: string, data?: ApiErrorData & { v1Error?: ShroudEditErrorResponse }) {
		// If we have a V1 error, format the message nicely
		let errorMessage = message
		if (data?.v1Error) {
			errorMessage = `[${data.v1Error.error}] ${data.v1Error.description}`
			if (data.v1Error.context) {
				errorMessage = `${data.v1Error.context}: ${errorMessage}`
			}
		}

		super(errorMessage, data)
		this.name = 'ShroudEditServerError'
		this.v1Error = data?.v1Error

		if (Error.captureStackTrace) {
			Error.captureStackTrace(this, ShroudEditServerError)
		}
	}

	/**
	 * Create a ShroudEditServerError from response data
	 */
	static fromResponse(
		statusCode: number,
		responseData: unknown,
		context?: string,
	): ShroudEditServerError {
		const v1Error = isShroudEditErrorResponse(responseData) ? responseData : undefined

		let message = `HTTP ${statusCode}`
		if (v1Error) {
			message = v1Error.description
		} else if (typeof responseData === 'string') {
			message = responseData
		}

		return new ShroudEditServerError(message, {
			statusCode,
			responseData,
			context,
			v1Error,
		})
	}

	/**
	 * Create a ShroudEditServerError from an unknown error
	 */
	static fromUnknown(error: unknown, context?: string): ShroudEditServerError {
		if (error instanceof ShroudEditServerError) {
			return error
		}

		if (error instanceof ShroudEditApiError) {
			return new ShroudEditServerError(error.message, {
				statusCode: error.statusCode,
				originalError: error.originalError,
				responseData: error.responseData,
				context: context ?? error.context,
			})
		}

		if (error instanceof Error) {
			return new ShroudEditServerError(error.message, {
				originalError: error,
				context,
			})
		}

		return new ShroudEditServerError(String(error), { context })
	}
}
