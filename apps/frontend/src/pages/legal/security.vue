<template>
	<div class="markdown-body">
		<h1>Security Notice</h1>
		<p><em>Last updated: September 20, 2026</em></p>

		<p>
			This notice explains how to report suspected security vulnerabilities affecting ShroudEdit,
			its API or the source code published at
			<a href="https://github.com/bosnbauer/shroudedit">github.com/bosnbauer/shroudedit</a>.
		</p>

		<h2>Reporting a vulnerability</h2>
		<p>
			Please report vulnerabilities privately to
			<a href="mailto:security@shroudedit.com">security@shroudedit.com</a>.
			<strong>Do not publish an unresolved vulnerability in a public issue.</strong>
		</p>
		<p>Please include, where available:</p>
		<ul>
			<li>the affected website, API endpoint, repository or component;</li>
			<li>a clear description of the vulnerability and its potential impact;</li>
			<li>minimal, non-destructive reproduction steps;</li>
			<li>relevant logs, screenshots or proof-of-concept material with secrets removed;</li>
			<li>a safe way to contact you about the report.</li>
		</ul>

		<h2>Responsible testing</h2>
		<p>
			A security report does not authorise access beyond what is necessary to demonstrate the issue.
			Do not access, alter, retain or disclose other users’ data; interrupt the Service; use social
			engineering; perform denial-of-service testing; deploy malware; or test third-party systems
			without their permission. Stop testing and contact us if you encounter personal data or obtain
			unexpected access.
		</p>

		<h2>Our response</h2>
		<p>
			We will aim to acknowledge useful reports, assess their severity and coordinate remediation and
			disclosure where appropriate. Response and remediation times depend on severity, complexity and
			available resources. We do not currently operate a paid bug-bounty programme.
		</p>

		<h2>Usually out of scope</h2>
		<ul>
			<li>automated scan output without a reproducible security impact;</li>
			<li>missing best-practice headers without an exploitable consequence;</li>
			<li>rate-limit or denial-of-service testing that risks service availability;</li>
			<li>issues exclusively affecting unsupported browsers or modified clients;</li>
			<li>vulnerabilities in third-party services that do not affect ShroudEdit.</li>
		</ul>

		<h2>Personal data incidents</h2>
		<p>
			If a report concerns personal data, also contact
			<a href="mailto:gdpr@shroudedit.com">gdpr@shroudedit.com</a>. Do not include unnecessary
			personal data in the report.
		</p>
	</div>
</template>

<script setup>
const description = 'How to report security vulnerabilities affecting ShroudEdit.'

useSeoMeta({
	title: 'Security Notice - ShroudEdit',
	description,
	ogTitle: 'Security Notice',
	ogDescription: description,
})
</script>
