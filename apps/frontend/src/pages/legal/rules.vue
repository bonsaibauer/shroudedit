<template>
	<div class="markdown-body">
		<h1>Content Rules</h1>
		<p><em>Last updated: September 20, 2026</em></p>

		<p>
			These Content Rules form part of the
			<nuxt-link to="/legal/terms">Terms of Use</nuxt-link> and apply to all projects, files,
			schematics, modpacks, server listings, images, descriptions, comments and other material shared
			through ShroudEdit (<strong>“Content”</strong>).
		</p>
		<p>
			Use the report button on the relevant page to report a violation. Reports of illegal content
			may also be sent to
			<a href="mailto:support@shroudedit.com">support@shroudedit.com</a>. Copyright reports are
			handled under our <nuxt-link to="/legal/copyright">Copyright Policy</nuxt-link>.
		</p>

		<h2 id="illegal-harmful-content">1. Illegal and harmful content</h2>
		<p>Content must comply with applicable law and must not:</p>
		<ol>
			<li>promote, facilitate or contain unlawful activity or unlawful material;</li>
			<li>
				infringe copyright, trademarks, privacy, personality rights, trade secrets or other rights;
			</li>
			<li>
				contain credible threats, targeted harassment, doxxing, incitement to violence or hateful
				content directed at protected groups;
			</li>
			<li>contain sexual exploitation, sexualised content involving minors or grooming behaviour;</li>
			<li>impersonate another person or falsely claim approval, authorship or affiliation;</li>
			<li>publish personal or confidential information without a lawful basis;</li>
			<li>deceive users about the nature, origin, safety or behaviour of a file or service.</li>
		</ol>

		<h2 id="technical-safety">2. Technical safety</h2>
		<p>Content must not:</p>
		<ol>
			<li>
				contain malware, ransomware, credential theft, destructive payloads, cryptominers or code
				designed to compromise a device, account, server or network;
			</li>
			<li>
				collect, transmit or expose personal data, files, identifiers or telemetry without clear prior
				disclosure and a lawful basis;
			</li>
			<li>download or execute undisclosed remote code or dependencies;</li>
			<li>circumvent ShroudEdit security controls, moderation or download accounting;</li>
			<li>interfere with Enshrouded servers or other users without their informed permission.</li>
		</ol>
		<p>
			Network access, telemetry, external executables and elevated permissions must be explained
			prominently on the project page. Disclosures must be accurate for every published version.
		</p>

		<h2 id="project-description">3. Clear and honest presentation</h2>
		<p>
			A project page must clearly explain what the project does, how it is installed or used, what
			game and loader versions it supports and any important limitations or risks. Titles, summaries,
			categories, dependencies, licences and environment tags must be accurate and must not be used
			to manipulate search or impersonate another project.
		</p>
		<ul>
			<li>Images and videos must represent the actual project.</li>
			<li>External links must be relevant, safe and publicly accessible where required.</li>
			<li>Required dependencies must be declared for the affected versions.</li>
			<li>Material differences in purpose should be published as a separate project.</li>
			<li>Descriptions must remain readable without relying exclusively on images.</li>
		</ul>

		<h2 id="content-types">4. Content-type requirements</h2>
		<h3>4.1. Mods</h3>
		<p>
			Mods must identify their supported Enshrouded versions and loader, including Shroudtopia,
			Shroudforge or EML where applicable. Any client/server requirements and configuration steps
			must be disclosed.
		</p>
		<h3>4.2. Schematics</h3>
		<p>
			Schematics must be genuine ShroudForge World Editor <code>.sfbp</code> content and must provide the
			relevant schematic format version, dimensions, compatible game and World Editor versions, and
			installation or import instructions. A schematic must not include a world or asset that the
			uploader is not permitted to redistribute.
		</p>
		<h3>4.3. Modpacks</h3>
		<p>
			Modpacks must respect the licences and redistribution permissions of included projects. Every
			required project or external file must be declared through the supported dependency and pack
			metadata mechanisms.
		</p>
		<h3>4.4. Servers</h3>
		<p>
			Server listings must describe the actual server, access requirements and enabled communication
			features. Passwords published for guest, friend or visitor roles are visible to users and must
			not be reused for administrative access or other services. Listings must not conceal paywalls,
			misrepresent player counts or redirect users to unrelated services.
		</p>

		<h2 id="copyright-reuploads">5. Copyright, licences and reuploads</h2>
		<p>
			You must own or have all licences, rights and permissions needed to publish and distribute your
			Content. Direct reuploads are prohibited unless authorised by the rightsholder or permitted by
			the applicable licence. Forks must comply with the original licence and clearly identify their
			source and changes. Third-party assets and code must be credited where required.
		</p>

		<h2 id="cheats-abuse">6. Cheats and abusive functionality</h2>
		<p>
			Projects designed primarily to gain an undisclosed unfair advantage, evade server rules,
			interfere with other players or bypass access controls are not permitted. Tools with legitimate
			administrative, accessibility, testing or single-player purposes must describe those purposes
			and include safeguards appropriate to their risk.
		</p>

		<h2 id="generative-ai">7. Generative AI</h2>
		<p>
			Material use of generative AI in code, assets, descriptions or project functionality must be
			disclosed accurately. AI-assisted Content remains subject to the same requirements for rights,
			quality, safety and truthful attribution. Content that is predominantly mass-generated,
			deceptive, infringing or published as spam may be rejected or removed.
		</p>

		<h2 id="moderation">8. Enforcement</h2>
		<p>
			Depending on severity and context, we may request changes, reject a submission, reduce
			visibility, remove files, suspend distribution or restrict an account. We consider intent,
			harm, recurrence, cooperation and the reliability of available evidence. Where required by law,
			we provide reasons and an opportunity to request review. Appeals may be sent to
			<a href="mailto:support@shroudedit.com">support@shroudedit.com</a>.
		</p>
	</div>
</template>

<script setup>
const description = 'The rules for content published and distributed through ShroudEdit.'

useSeoMeta({
	title: 'Content Rules - ShroudEdit',
	description,
	ogTitle: 'Content Rules',
	ogDescription: description,
})
</script>

<style scoped>
.markdown-body h2 {
	@apply mb-3 mt-5 pb-1 pt-0 text-xl font-semibold text-contrast;
}
.markdown-body h3 {
	@apply mb-1 mt-5 p-0 text-lg font-semibold text-contrast;
}

.markdown-body a {
	@apply !font-semibold !text-brand !no-underline hover:!text-brand hover:!underline;
}

.markdown-body p {
	@apply leading-normal;
}

.markdown-body ol,
.markdown-body ul {
	@apply mb-3;
}

.markdown-body li {
	@apply mb-1;
}
</style>
