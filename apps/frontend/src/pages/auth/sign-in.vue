<template>
	<SignInView
		v-model:email="email"
		v-model:password="password"
		v-model:token="token"
		v-model:two-factor-code="twoFactorCode"
		:flow="flow"
		:redirect-target="redirectTarget"
		:route-query="route.query"
		:globals="globals"
		:on-password-sign-in="beginPasswordSignIn"
		:on-two-factor-sign-in="begin2FASignIn"
		:two-factor-pending="twoFactorPending"
		:two-factor-error="twoFactorError"
		:on-passkey-sign-in="beginPasskeySignin"
		:on-set-captcha-ref="setCaptchaRef"
	/>
</template>

<script setup lang="ts">
import {
	commonMessages,
	defineMessages,
	injectShroudEditClient,
	injectNotificationManager,
	useVIntl,
} from '@shroudedit/ui'
import { useQuery, useQueryClient } from '@tanstack/vue-query'
import { useStorage } from '@vueuse/core'
import type { LocationQueryValue } from 'vue-router'

import SignInView from '@/components/ui/auth/SignIn.vue'
import {
	isStoredAccountAuthMethod,
	LAST_SIGN_IN_OAUTH_PROVIDER_STORAGE_KEY,
	PENDING_SIGN_IN_OAUTH_PROVIDER_STORAGE_KEY,
	rememberStoredAccount,
	type StoredAccountAuthMethod,
} from '@/composables/accounts.ts'
import { promotePendingSignInOAuthProvider } from '@/composables/auth.ts'
import { getPasskeyCredential } from '@/helpers/passkey.ts'

type AuthProvider = 'discord' | 'google' | 'github' | 'gitlab' | 'steam' | 'microsoft' | 'passkey'

interface AuthGlobalsResponse {
	captcha_enabled?: boolean
	[key: string]: unknown
}

interface ApiErrorShape {
	data?: {
		description?: string
	}
}

const getQueryString = (
	value: LocationQueryValue | LocationQueryValue[] | null | undefined,
): string => {
	const firstValue = Array.isArray(value) ? value[0] : value
	return typeof firstValue === 'string' ? firstValue : ''
}

const getErrorMessage = (error: unknown): string => {
	const apiError = error as ApiErrorShape
	if (typeof apiError?.data?.description === 'string') {
		return apiError.data.description
	}
	if (error instanceof Error) {
		return error.message
	}
	return String(error)
}

const client = injectShroudEditClient()
const queryClient = useQueryClient()
const { addNotification } = injectNotificationManager()
const { formatMessage } = useVIntl()

const messages = defineMessages({
	signInTitle: {
		id: 'auth.sign-in.title',
		defaultMessage: 'Sign In',
	},
})

useHead({
	title() {
		return `${formatMessage(messages.signInTitle)} - ShroudEdit`
	},
})

const route = useNativeRoute()
const pendingSignInOAuthProvider = useStorage<AuthProvider | null>(
	PENDING_SIGN_IN_OAUTH_PROVIDER_STORAGE_KEY,
	null,
	undefined,
	{ initOnMounted: true },
)
const lastSignInOAuthProvider = useStorage<AuthProvider | null>(
	LAST_SIGN_IN_OAUTH_PROVIDER_STORAGE_KEY,
	null,
	undefined,
	{ initOnMounted: true },
)

if (route.query.state !== undefined) {
	await navigateTo(
		{
			path: '/auth/create/oauth',
			query: route.query,
		},
		{
			replace: true,
		},
	)
}

const redirectTarget = getQueryString(route.query.redirect)

if (route.query.code) {
	await finishSignIn()
}

const captcha = ref<{ reset?: () => void } | null>(null)
const setCaptchaRef = (captchaRef: unknown) => {
	captcha.value = (captchaRef as { reset?: () => void } | null) ?? null
}

const { data: globals } = useQuery<AuthGlobalsResponse>({
	queryKey: ['auth-globals'],
	queryFn: async () => {
		try {
			return await client.labrinth.globals_internal.get()
		} catch (err) {
			console.error('Error fetching globals:', err)
			return { captcha_enabled: true, tax_compliance_thresholds: {} }
		}
	},
})

const email = ref('')
const password = ref('')
const token = ref('')

const flow = ref(getQueryString(route.query.flow))

async function beginPasswordSignIn() {
	pendingSignInOAuthProvider.value = null
	lastSignInOAuthProvider.value = null
	startLoading()
	try {
		const res = await client.labrinth.auth_v2.login({
			username: email.value,
			password: password.value,
			challenge: token.value,
		})

		if (res.flow) {
			flow.value = res.flow
		} else {
			await finishSignIn(res.session, 'password')
		}
	} catch (err) {
		addNotification({
			title: formatMessage(commonMessages.errorNotificationTitle),
			text: getErrorMessage(err),
			type: 'error',
		})
		captcha.value?.reset?.()
	}
	stopLoading()
}

const twoFactorCode = ref('')
const twoFactorPending = ref(false)
const twoFactorError = ref(false)

async function begin2FASignIn(code: string) {
	if (twoFactorPending.value) return
	twoFactorPending.value = true
	twoFactorError.value = false
	startLoading()
	try {
		const res = await client.labrinth.auth_v2.login2FA({
			flow: flow.value,
			code,
		})

		await finishSignIn(res.session, 'password')
	} catch {
		twoFactorCode.value = ''
		twoFactorError.value = true
	} finally {
		twoFactorPending.value = false
		stopLoading()
	}
}

async function beginPasskeySignin() {
	startLoading()
	try {
		const start = await client.labrinth.auth_v2.authenticatePasskeyStart()

		const credential = await getPasskeyCredential(start.options.publicKey)

		const result = await client.labrinth.auth_v2.authenticatePasskeyFinish({
			flow: start.flow,
			credential,
		})

		pendingSignInOAuthProvider.value = 'passkey'
		await finishSignIn(result.session, 'passkey')
	} catch (err) {
		addNotification({
			title: formatMessage(commonMessages.errorNotificationTitle),
			text: getErrorMessage(err),
			type: 'error',
		})
	}
	stopLoading()
}

async function finishSignIn(sessionToken?: string | null, authMethod?: StoredAccountAuthMethod) {
	if (sessionToken) {
		await useAuth(sessionToken)
		await useUser()
		queryClient.clear()
	}

	const signedIn = await useAuth()
	if (signedIn.value.user && signedIn.value.token) {
		const nextAuthMethod =
			authMethod ??
			(isStoredAccountAuthMethod(pendingSignInOAuthProvider.value)
				? pendingSignInOAuthProvider.value
				: undefined)
		rememberStoredAccount(
			signedIn.value.user,
			signedIn.value.token,
			nextAuthMethod ? { authMethod: nextAuthMethod } : undefined,
		)
	}

	promotePendingSignInOAuthProvider()

	if (route.query.redirect) {
		const redirect = decodeURIComponent(getQueryString(route.query.redirect))
		await navigateTo(redirect, {
			replace: true,
		})
	} else if (signedIn.value.user) {
		await navigateTo(`/user/${signedIn.value.user.username}`)
	}
}
</script>
