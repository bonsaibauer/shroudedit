import { getRequestHeaders, getRequestURL, type H3Event } from 'h3'

import { readEnv } from '~/helpers/env'

const CDN_HOST = 'cdn.shroudedit.com'
const DEV_DOWNLOAD_HOST = 'dev.shroudedit.com'
const VERSION_FILE_PATH = /^\/data\/([^/]+)\/versions\/([^/]+)\/.+$/

type CloudflareEventContext = {
	cloudflare?: {
		context?: {
			waitUntil?: (promise: Promise<unknown>) => void
		}
	}
}

export default defineEventHandler(async (event) => {
	const requestUrl = getRequestURL(event)

	const isDevDownload = requestUrl.hostname === DEV_DOWNLOAD_HOST
	if (
		(requestUrl.hostname !== CDN_HOST && !isDevDownload) ||
		!requestUrl.pathname.startsWith('/data/')
	) {
		return
	}

	if (!['GET', 'HEAD', 'OPTIONS'].includes(event.method)) {
		return new Response('Method Not Allowed', {
			status: 405,
			headers: { Allow: 'GET, HEAD, OPTIONS' },
		})
	}

	const requestHeaders = new Headers(getRequestHeaders(event))
	requestHeaders.delete('authorization')
	requestHeaders.delete('cookie')
	requestHeaders.delete('host')

	const originUrl = new URL(requestUrl)
	if (isDevDownload) {
		originUrl.hostname = CDN_HOST
	}

	const originResponse = await fetch(
		new Request(originUrl, {
			method: event.method,
			headers: requestHeaders,
		}),
	)

	const versionFilePath = requestUrl.pathname.match(VERSION_FILE_PATH)
	if (event.method === 'GET' && originResponse.ok && versionFilePath) {
		const trackingUrl = new URL(requestUrl)
		trackingUrl.hostname = CDN_HOST
		const countPromise = countDownload(event, trackingUrl, versionFilePath).catch((error) => {
			console.error('[cdn-download-tracking] failed to count download', error)
		})
		const cloudflareContext =
			(event.context as typeof event.context & CloudflareEventContext).cloudflare?.context

		if (cloudflareContext?.waitUntil) {
			cloudflareContext.waitUntil(countPromise)
		} else {
			await countPromise
		}
	}

	return originResponse
})

async function countDownload(
	event: H3Event,
	requestUrl: URL,
	versionFilePath: RegExpMatchArray,
) {
	const adminKey = await readEnv('LABRINTH_ADMIN_KEY')
	const apiBaseUrl = await readEnv('BASE_URL')
	if (!adminKey || !apiBaseUrl) {
		throw new Error('download tracking credentials are not configured')
	}

	const incomingHeaders = new Headers(getRequestHeaders(event))
	const ip = incomingHeaders.get('cf-connecting-ip') ?? '127.0.0.1'
	for (const header of ['authorization', 'cookie', 'shroudedit-admin']) {
		incomingHeaders.delete(header)
	}

	const baseUrl = apiBaseUrl.endsWith('/') ? apiBaseUrl : `${apiBaseUrl}/`
	const response = await fetch(new URL('admin/_count-download', baseUrl), {
		method: 'PATCH',
		headers: {
			'content-type': 'application/json',
			'ShroudEdit-Admin': adminKey,
		},
		body: JSON.stringify({
			url: requestUrl.toString(),
			project_id: versionFilePath[1],
			version_name: versionFilePath[2],
			ip,
			headers: Object.fromEntries(incomingHeaders.entries()),
		}),
	})

	if (!response.ok) {
		throw new Error(`Labrinth returned ${response.status} while counting a download`)
	}
}
