<template>
	<div class="markdown-body">
		<h1>Imprint</h1>
		<p>
			This legal notice applies to the Service as defined in the
			<nuxt-link to="/legal/terms">Terms of Use</nuxt-link>.
		</p>

		<h2>Provider information</h2>
		<p>
			<strong>Operator:</strong> Philipp Feierabend<br />
			c/o IMPREGO<br />
			Golzwarder Straße 28<br />
			26919 Brake (Unterweser)<br />
			Germany
		</p>

		<h2>Contact</h2>
		<p>
			Email: <a href="mailto:support@shroudedit.com">support@shroudedit.com</a><br />
			Email: support [at] shroudedit [dot] com
		</p>

		<h2>Responsible for editorial content</h2>
		<p>
			Philipp Feierabend<br />
			c/o IMPREGO<br />
			Golzwarder Straße 28<br />
			26919 Brake (Unterweser)<br />
			Germany
		</p>

		<h2>Platform contact points</h2>
		<p>
			General and legal notices:
			<a href="mailto:support@shroudedit.com">support@shroudedit.com</a><br />
			Copyright notices:
			<a href="mailto:copyright@shroudedit.com">copyright@shroudedit.com</a><br />
			Data protection:
			<a href="mailto:privacy@shroudedit.com">privacy@shroudedit.com</a><br />
			Security reports:
			<a href="mailto:security@shroudedit.com">security@shroudedit.com</a>
		</p>
		<p>Communication is possible in German and English.</p>

		<h2>Trademark notice</h2>
		<p>
			NOT AN OFFICIAL ENSHROUDED SERVICE. NOT APPROVED BY OR ASSOCIATED WITH KEEN GAMES. Enshrouded
			and related names and marks belong to their respective owners.
		</p>
	</div>
</template>

<script setup>
const description = 'Legal provider information and contact details for the Service.'

useSeoMeta({
	title: 'Imprint - ShroudEdit',
	description,
	ogTitle: 'Imprint',
	ogDescription: description,
})
</script>
