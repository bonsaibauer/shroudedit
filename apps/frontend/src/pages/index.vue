<template>
	<div>
		<div class="landing-hero">
			<ShroudEditIcon class="brand-icon" />
			<h1 class="main-header">
				<IntlFormatted :message-id="messages.thePlaceForEnshrouded">
					<template #~content>
						<div class="animate-strong">
							<span>
								<strong
									v-for="[key, message] in Object.entries(contentTypeMessages)"
									:key="`landing-content-type-${key}`"
									class="main-header-strong"
								>
									{{ formatMessage(message) }} <br />
								</strong>
								<strong class="main-header-strong">
									{{ formatMessage(contentTypeMessages.mods) }}
								</strong>
							</span>
						</div>
					</template>
				</IntlFormatted>
			</h1>
			<h2>
				{{ formatMessage(messages.discoverHeading) }}
			</h2>
			<div class="mx-auto mb-20 flex w-fit flex-wrap justify-center gap-5">
				<ButtonLink type="colored" color="brand" size="xl" to="/discover/mods">
					<CompassIcon aria-hidden="true" />
					{{ formatMessage(messages.discoverMods) }}
				</ButtonLink>
				<ButtonLink
					v-if="!auth.user"
					type="outlined"
					size="xl"
					to="/auth/sign-up"
					rel="noopener nofollow"
				>
					<LogInIcon aria-hidden="true" />
					{{ formatMessage(commonMessages.signUpButton) }}
				</ButtonLink>
				<ButtonLink v-else type="outlined" size="xl" to="/dashboard/projects">
					<DashboardIcon aria-hidden="true" />
					{{ formatMessage(messages.goToDashboard) }}
				</ButtonLink>
			</div>
		</div>
		<div class="users-section-outer">
			<div v-if="rows" class="projects-showcase">
				<div v-for="(row, index) in rows" :key="index" class="row">
					<div v-for="n in 2" :key="n" class="row__content" :class="{ offset: index % 2 }">
						<ButtonLink
							v-for="project in row"
							:key="project.id"
							type="quiet"
							interaction="none"
							class="project !h-auto !shrink !items-stretch !justify-start !whitespace-normal !p-4 !font-normal !leading-normal"
							:to="`/${project.project_type}/${project.slug ? project.slug : project.id}`"
						>
							<Avatar :src="project.icon_url" :alt="project.title" size="sm" loading="lazy" />
							<div class="project-info">
								<span class="title">
									{{ project.title }}
								</span>
								<span class="description">
									{{ project.description }}
								</span>
							</div>
						</ButtonLink>
					</div>
				</div>
			</div>
			<div v-else class="relative z-[10] w-full text-center text-xl font-bold text-contrast">
				{{ formatMessage(messages.failedToLoadRandomProjects) }}
			</div>
			<div class="projects-transition" />
			<div class="users-section">
				<div class="section-header">
					<div class="section-label green">{{ formatMessage(messages.forPlayersLabel) }}</div>
					<h2 class="section-tagline">
						{{ formatMessage(messages.discoverCreationsTagline, { count: PROJECT_COUNT }) }}
					</h2>
					<p class="section-description">
						{{ formatMessage(messages.playersDescription) }}
					</p>
				</div>
				<div class="feature-blob">
					<div class="blob-text">
						<h3>{{ formatMessage(messages.findWhatYouWantHeading) }}</h3>
						<p>
							{{ formatMessage(messages.findWhatYouWantDescription) }}
						</p>
					</div>
					<div class="blob-demonstration gradient-border bigger">
						<div class="demo-search">
							<div class="search-controls">
								<label class="hidden" for="search">{{
									formatMessage(commonMessages.searchLabel)
								}}</label>
								<Input
									id="search"
									v-model="searchQuery"
									:icon="SearchIcon"
									type="search"
									name="search"
									:placeholder="formatMessage(commonMessages.searchPlaceholder)"
									autocomplete="off"
									wrapper-class="w-full"
									@input="updateSearchProjects"
								/>
								<div class="sort-by">
									<DropdownSelect
										v-slot="{ selected }"
										v-model="sortType"
										class="!h-9 !w-max flex-grow"
										name="Sort by"
										:options="sortOptions"
										:display-name="(value) => value?.charAt(0).toUpperCase() + value?.slice(1)"
										@change="updateSearchProjects()"
									>
										<div>
											<span class="font-semibold text-primary"
												>{{ formatMessage(commonMessages.sortByLabel) }}:
											</span>
											<span class="font-semibold text-secondary">{{ selected }}</span>
										</div>
									</DropdownSelect>
								</div>
							</div>
							<div class="results display-mode--list">
								<ProjectCard
									v-for="result in searchProjects"
									:key="result.project_id"
									class="gradient-border"
									:link="`/${result.project_type}/${result.slug ? result.slug : result.project_id}`"
									:title="result.title"
									:author="{ name: result.author, link: `/user/${result.author}` }"
									:summary="result.description"
									:date-updated="result.date_modified"
									:date-published="result.date_created"
									:displayed-date="sortType === 'newest' ? 'published' : 'updated'"
									:downloads="result.downloads"
									:followers="result.follows"
									:icon-url="result.icon_url"
									:environment="{
										clientSide: result.client_side,
										serverSide: result.server_side,
									}"
									:tags="result.display_categories.slice(0, 3)"
									:color="result.color"
									layout="list"
								/>
							</div>
						</div>
					</div>
				</div>
				<div class="feature-blob reverse">
					<div class="blob-text">
						<h3>{{ formatMessage(messages.followProjectsHeading) }}</h3>
						<p>{{ formatMessage(messages.followProjectsDescription) }}</p>
					</div>
					<div class="blob-demonstration gradient-border">
						<div class="notifs-demo">
							<h3>{{ formatMessage(commonMessages.notificationsLabel) }}</h3>
							<div class="notifications">
								<div
									v-for="(notification, index) in notifications"
									:key="index"
									class="notification gradient-border"
								>
									<nuxt-link
										:to="`${notification.project_type}/${notification.slug}`"
										:title="notification.title"
									>
										<Avatar size="md" :src="notification.icon_url" :alt="notification.title" />
									</nuxt-link>
									<div>
										<nuxt-link
											:to="`${notification.project_type}/${notification.slug}`"
											class="notif-header"
										>
											{{ formatMessage(messages.hasBeenUpdated, { title: notification.title }) }}
										</nuxt-link>
										<p class="notif-desc">
											{{
												formatMessage(messages.versionReleased, {
													version: ['1.1.2', '1.0.3', '15.1'][index],
													gameVersion: notification.versions[notification.versions.length - 1],
												})
											}}
										</p>
										<div class="date">
											<CalendarIcon aria-hidden="true" />
											<span>
												{{
													formatMessage(messages.receivedTime, {
														time: formatRelativeTime(notification.date_modified),
													})
												}}
											</span>
										</div>
									</div>
								</div>
							</div>
						</div>
					</div>
				</div>
				<div class="feature-blob">
					<div class="blob-text">
						<h3>{{ formatMessage(messages.playWithLauncherHeading) }}</h3>
						<p>
							<IntlFormatted :message-id="messages.playWithLauncherDescription">
								<template #link="{ children }">
									<nuxt-link
										class="underline hover:brightness-[--hover-brightness]"
										to="/discover/schematics"
									>
										<component :is="() => children" />
									</nuxt-link>
								</template>
							</IntlFormatted>
						</p>
					</div>
					<div class="blob-demonstration gradient-border">
						<div class="launcher-view">
							<QuestionMarkIcon class="question-mark-placeholder" aria-hidden="true" />
						</div>
					</div>
				</div>
			</div>
		</div>
		<div class="bg-[var(--landing-raw-bg)]">
			<LatestNewsRow />
		</div>
	</div>
</template>
<script setup>
import {
	CalendarIcon,
	CompassIcon,
	DashboardIcon,
	LogInIcon,
	QuestionMarkIcon,
	ShroudEditIcon,
	SearchIcon,
} from '@shroudedit/assets'
import {
	Avatar,
	ButtonLink,
	commonMessages,
	defineMessages,
	DropdownSelect,
	injectShroudEditClient,
	Input,
	IntlFormatted,
	ProjectCard,
	useRelativeTime,
	useVIntl,
} from '@shroudedit/ui'
import { ref } from 'vue'

import LatestNewsRow from '~/components/ui/news/LatestNewsRow.vue'
import { homePageNotifs, homePageProjects, homePageSearch } from '~/generated/state.json'

const formatRelativeTime = useRelativeTime()

const { formatMessage } = useVIntl()
const { labrinth } = injectShroudEditClient()
const config = useRuntimeConfig()

const searchQuery = ref('leave')
const sortType = ref('relevance')
const sortOptions = ['relevance', 'downloads', 'follows', 'updated', 'newest']

const PROJECT_COUNT = 100000

const auth = await useAuth()

const newProjects = homePageProjects?.slice(0, 40) ?? []
const val = Math.ceil(newProjects.length / 3)
const rows = ref(
	newProjects.length > 0
		? [
				newProjects.slice(0, val),
				newProjects.slice(val, val * 2),
				newProjects.slice(val * 2, val * 3),
			]
		: undefined,
)

const notifications = ref(homePageNotifs?.hits ?? [])
const searchProjects = ref(homePageSearch?.hits ?? [])

async function updateSearchProjects() {
	searchProjects.value = await getSearchProjects()
}

async function getSearchProjects() {
	try {
		const res = await labrinth.projects_v2.search({
			limit: 3,
			query: searchQuery.value,
			index: sortType.value,
		})

		return res?.hits ?? []
	} catch {
		return []
	}
}

if (searchProjects.value.length === 0) {
	searchProjects.value = await getSearchProjects()
}

const messages = defineMessages({
	thePlaceForEnshrouded: {
		id: 'landing.heading.the-place-for-enshrouded',
		defaultMessage: 'The place for Enshrouded {content}',
	},
	discoverHeading: {
		id: 'landing.subheading.enshrouded',
		defaultMessage:
			'Discover, use, and share Enshrouded content through our open-source platform built for the community.',
	},
	discoverMods: {
		id: 'landing.button.discover-mods',
		defaultMessage: 'Discover mods',
	},
	goToDashboard: {
		id: 'landing.button.go-to-dashboard',
		defaultMessage: 'Go to dashboard',
	},
	failedToLoadRandomProjects: {
		id: 'landing.empty.projects.enshrouded',
		defaultMessage: 'Community projects will appear here as they are published.',
	},
	forPlayersLabel: {
		id: 'landing.section.for-players.label',
		defaultMessage: 'For Players',
	},
	discoverCreationsTagline: {
		id: 'landing.section.for-players.tagline.enshrouded',
		defaultMessage: 'Discover Enshrouded community creations',
	},
	playersDescription: {
		id: 'landing.section.for-players.description',
		defaultMessage:
			'From magical biomes to cursed dungeons, you can be sure to find content to bring your gameplay to the next level.',
	},
	findWhatYouWantHeading: {
		id: 'landing.feature.search.heading',
		defaultMessage: 'Find what you want, quickly and easily',
	},
	findWhatYouWantDescription: {
		id: 'landing.feature.search.description',
		defaultMessage:
			"ShroudEdit's lightning-fast search and powerful filters let you find what you want as you type.",
	},
	followProjectsHeading: {
		id: 'landing.feature.follow.heading',
		defaultMessage: 'Follow projects you love',
	},
	followProjectsDescription: {
		id: 'landing.feature.follow.description',
		defaultMessage: 'Get notified every time your favorite projects update and stay in the loop.',
	},
	playWithLauncherHeading: {
		id: 'landing.feature.tools.heading.enshrouded',
		defaultMessage: 'Connect your favorite community tools',
	},
	playWithLauncherDescription: {
		id: 'landing.feature.tools.description.enshrouded',
		defaultMessage:
			"ShroudEdit's open-source API lets community tools add deep integration. You can also <link>discover and share World Editor schematics</link> through the same platform.",
	},
	notificationsHeading: {
		id: 'landing.notifications.heading',
		defaultMessage: 'Notifications',
	},
	hasBeenUpdated: {
		id: 'landing.notifications.has-been-updated',
		defaultMessage: '{title} has been updated!',
	},
	versionReleased: {
		id: 'landing.notifications.version-released',
		defaultMessage: 'Version {version} has been released for {gameVersion}',
	},
	receivedTime: {
		id: 'landing.notifications.received-time',
		defaultMessage: 'Received {time}',
	},
})

const contentTypeMessages = defineMessages({
	mods: {
		id: 'landing.heading.the-place-for-enshrouded.mods',
		defaultMessage: 'mods',
	},
	schematics: {
		id: 'landing.heading.the-place-for-enshrouded.schematics',
		defaultMessage: 'schematics',
	},
	modpacks: {
		id: 'landing.heading.the-place-for-enshrouded.modpacks',
		defaultMessage: 'modpacks',
	},
	servers: {
		id: 'landing.heading.the-place-for-enshrouded.servers',
		defaultMessage: 'servers',
	},
})

</script>

<style lang="scss" scoped>
.landing-hero {
	background-image: var(--landing-maze-bg);
	background-size: cover;
	object-fit: contain;
	padding: 6rem 1rem 12rem 1rem;

	display: flex;
	justify-content: center;
	align-items: center;
	text-align: center;
	flex-direction: column;

	.brand-icon {
		width: 13rem;
		height: 13rem;
		margin-bottom: 2.5rem;
	}

	h2 {
		font-size: 1.25rem;
		line-height: 125%;
		margin: 0 0 1.625rem;
		font-weight: 400;
		line-break: loose;
		color: var(--landing-color-subheading);
		max-width: 50rem;
	}
}

.users-section-outer {
	position: relative;
	background: var(--landing-maze-outer-bg);
	width: 100%;

	&:before {
		content: '';
		position: absolute;
		z-index: 1;
		inset: 0;
		background: linear-gradient(
			180deg,
			var(--landing-transition-gradient-end) 0%,
			var(--landing-transition-gradient-start) 100%
		);
		height: 12.5rem;
		width: 100%;
	}

	.projects-transition {
		position: absolute;
		top: calc(-12.5rem);
		width: 100%;
		height: 12.5rem;
		background: linear-gradient(
			0deg,
			var(--landing-transition-gradient-end) 0%,
			var(--landing-transition-gradient-start) 100%
		);
	}

	.projects-showcase {
		position: absolute;
		z-index: 2;
		top: -11rem;

		.row {
			--gap: 1.5rem;

			width: 100vw;
			gap: var(--gap);
			margin-bottom: var(--gap);

			display: flex;
			overflow: hidden;
			user-select: none;

			&:hover {
				.row__content {
					animation-play-state: paused !important;
				}
			}

			.row__content {
				flex-shrink: 0;
				display: flex;
				min-width: 100%;
				gap: var(--gap);
				animation: scroll 40s linear infinite;

				@media (prefers-reduced-motion) {
					animation-play-state: paused !important;
				}

				@keyframes scroll {
					from {
						transform: translateX(0);
					}
					to {
						transform: translateX(calc(-100%));
					}
				}

				&.offset {
					animation: scroll-inverse 40s linear infinite;
					transform: translateX(-100%);

					@keyframes scroll-inverse {
						from {
							transform: translateX(calc(-100%));
						}
						to {
							transform: translateX(calc(0%));
						}
					}
				}
			}

			.project {
				position: relative;
				display: flex;

				cursor: pointer;
				padding: 1rem;
				gap: 1rem;
				border-radius: 1rem;
				border: 1px solid var(--landing-border-color);
				transition:
					background 0.5s ease-in-out,
					transform 0.05s ease-in-out;
				// Removed due to lag on mobile :(

				&:hover {
					z-index: -2;
					background: var(--landing-hover-card-gradient);
				}

				img {
					height: 3rem;
				}

				.project-info {
					box-sizing: border-box;
				}

				.title {
					color: var(--landing-color-heading);
					max-width: 13.75rem;
					overflow: hidden;
					white-space: nowrap;
					text-overflow: ellipsis;
					margin: 0;
					font-weight: 600;
					font-size: 1.25rem;
					line-height: 110%;
					display: block;
				}

				.description {
					width: 13.75rem;

					display: -webkit-box;
					-webkit-line-clamp: 2;
					line-clamp: 2;
					-webkit-box-orient: vertical;
					overflow: hidden;

					font-weight: 500;
					font-size: 0.875rem;
					line-height: 125%;
					margin: 0.25rem 0 0;
				}
			}
		}
	}

	.users-section {
		width: 100%;
		padding-top: 10rem;
		padding-bottom: 5rem;

		background: var(--landing-maze-gradient-bg);
		background-size: cover;
		background-blend-mode: multiply;

		.feature-blob {
			display: flex;
			padding: 1.25rem 1rem;
			justify-content: center;
			flex-wrap: wrap;
			column-gap: 4.375rem;
			margin: 0 0.75rem 0.75rem 0.75rem;

			&.reverse {
				flex-direction: row-reverse;
			}

			.blob-text {
				margin-top: 2.5rem;
				width: 32.5rem;
				max-width: 32.5rem;
				text-align: center;

				h3 {
					font-weight: 500;
					font-size: 2rem;
					line-height: 110%;
					color: var(--landing-color-heading);
				}

				p {
					font-weight: 400;
					font-size: 1.25rem;
					line-height: 125%;
					color: var(--landing-color-subheading);
					line-break: loose;
				}
			}

			@media screen and (min-width: 1238px) {
				padding: 5rem 1rem;

				.blob-text {
					margin-top: 5rem;
					text-align: left;
				}
			}

			.blob-demonstration {
				position: relative;
				width: 35rem;
				max-width: 35rem;
				background: var(--landing-blob-gradient);
				box-shadow: var(--landing-blob-shadow);
				// backdrop-filter: blur(6px);
				background-blend-mode: multiply;
				padding: 1rem;
				z-index: 1;

				&:after {
					content: '';
					position: absolute;
					z-index: -1;
					inset: 0 0 -0.75rem -0.75rem;

					background: linear-gradient(0deg, #63c9f5 0%, rgba(99, 201, 245, 0) 100%);
					opacity: 0.2;
					border-radius: 1rem;
					margin-top: auto;
					width: calc(100% + 1.5rem);
					height: 55%;
				}

				.demo-search {
					height: max-content;

					.search-controls {
						display: flex;
						justify-content: space-between;
						margin-bottom: 1rem;
						gap: 0.5rem;

						.iconified-input {
							width: 100%;

							svg {
								opacity: 1;
								color: var(--color-button-text-active);
							}

							input {
								box-shadow:
									inset 0 0 0 transparent,
									0 0 0 0.25rem var(--color-brand-shadow);
								color: var(--color-button-text-active);
							}
						}

						.sort-by {
							display: flex;
							gap: 0.75rem;
							align-items: center;
						}
					}

					.results {
						display: flex;
						flex-direction: column;
						gap: 1rem;

						.small-mode {
							background: var(--landing-card-bg);
							box-shadow: var(--landing-card-shadow);
							background-blend-mode: multiply;
							// backdrop-filter: blur(4px);
						}

						@media screen and (max-width: 450px) {
							.small-mode:nth-child(n + 2) {
								display: none;
							}
						}

						@media screen and (max-width: 500px) {
							.small-mode:nth-child(n + 3) {
								display: none;
							}
						}
					}
				}

				.notifs-demo {
					h3 {
						font-weight: 600;
						font-size: 1.5rem;
						margin: 0 0 0.75rem;
					}

					.notifications {
						display: flex;
						flex-direction: column;
						gap: 1rem;

						.notification {
							display: flex;
							gap: 1rem;
							padding: 0.75rem;
							background: var(--landing-card-bg);
							box-shadow: var(--landing-card-shadow);

							img {
								width: 4rem;
								height: 4rem;
							}

							.notif-header {
								margin: 0;
								font-weight: 600;
								font-size: 1.25rem;
							}

							.notif-desc {
								margin: 0.5rem 0;
								font-weight: 500;
								font-size: 1rem;
							}

							.date {
								display: flex;
								align-items: center;
								gap: 0.25rem;

								span {
									font-size: 12px;
									font-weight: 700;
								}
							}
						}
					}
				}

				.launcher-view {
					display: flex;
					min-height: 20rem;
					align-items: center;
					justify-content: center;
				}

				.question-mark-placeholder {
					width: min(64%, 18rem);
					height: auto;
					filter: drop-shadow(0 1rem 2rem rgba(199, 138, 255, 0.2));
				}
			}
		}
	}
}

.logo-banner {
	position: relative;
	display: flex;
	align-items: center;
	justify-content: center;
	background: var(--landing-raw-bg);
	padding: 1rem 1rem 2rem 1rem;
	overflow: hidden;

	.brand-icon {
		z-index: 2;
		width: auto;
		height: 32rem;
	}

	.overlay {
		z-index: 3;
		position: absolute;
		bottom: 8rem;
		display: flex;
		flex-direction: column;
		align-items: center;
		text-align: center;
		gap: 1.5rem;
	}
}

.gradient-border {
	position: relative;
	border-radius: 1rem;

	&:before {
		content: '';
		position: absolute;
		inset: 0;
		padding: 1px;
		z-index: -1;
		border-radius: 1rem;
		background: var(--landing-border-gradient);

		-webkit-mask:
			linear-gradient(#fff 0 0) content-box,
			linear-gradient(#fff 0 0);
		mask:
			linear-gradient(#fff 0 0) content-box,
			linear-gradient(#fff 0 0);
		-webkit-mask-composite: xor;
		mask-composite: exclude;
	}
}

.section-header {
	text-align: center;
	margin: 2rem;

	.section-label {
		margin: 1.5rem auto;
		width: fit-content;
		padding: 1rem 1.5rem;
		border-radius: 0.5rem;
		font-weight: 700;
		font-size: 1rem;
		line-height: 125%;

		&.green {
			background: var(--landing-green-label-bg);
			color: var(--landing-green-label);
		}

		&.blue {
			background: var(--landing-blue-label-bg);
			color: var(--landing-blue-label);
		}
	}

	.section-tagline,
	.section-description {
		font-weight: 400;
		font-size: 1.25rem;
		line-break: loose;
		line-height: 125%;
		max-width: 50rem;
	}

	.section-tagline {
		margin: 0 auto;
		color: var(--landing-color-heading);
	}

	.section-description {
		margin: 0.375rem auto;
		color: var(--landing-color-subheading);
	}
}

.main-header {
	color: var(--landing-color-heading);
	font-size: 2.25rem;
	font-weight: 600;
	line-height: 100%;
	margin: 0 0 0.25rem;
	width: 100%;
}

.main-header-strong {
	font-weight: 600;
	background-color: #18344d;
	background-image: linear-gradient(180deg, #f8ffff 0%, #63c9f5 60%);
	background-size: 100%;
	background-clip: text;
	-webkit-text-fill-color: transparent;
	-moz-text-fill-color: transparent;
	color: transparent;
	white-space: nowrap;
}

.animate-strong {
	height: 1.2em;
	line-height: 120%;
	position: relative;
	overflow: hidden;
	display: flex;
	justify-content: center;

	> span {
		position: absolute;
		top: 0;
		animation: slide 14s infinite;

		@media (prefers-reduced-motion) {
			animation-play-state: paused !important;
		}
	}

	@keyframes slide {
		0%,
		12.5% {
			top: 0;
		}
		14.3%,
		26.8% {
			top: -1.2em;
		}
		28.6%,
		41.1% {
			top: -2.4em;
		}
		42.9%,
		55.4% {
			top: -3.6em;
		}
		57.2%,
		69.7% {
			top: -4.8em;
		}
		71.5%,
		84% {
			top: -6em;
		}
		85.8%,
		98.3% {
			top: -7.2em;
		}
		100% {
			top: -8.4em;
		}
	}
}

@media screen and (min-width: 560px) {
	.landing-hero {
		h2 {
			font-size: 1.5rem;
		}
	}

	.users-section-outer {
		.users-section {
			.feature-blob {
				.blob-text {
					h3 {
						font-size: 3rem;
					}

					p {
						font-size: 1.5rem;
					}
				}

				.blob-demonstration {
					.demo-search {
						min-height: 37.5rem;
					}

					.notifs-demo {
						.notifications .notification .avatar {
							width: 5rem;
							height: 5rem;
						}
					}
				}
			}
		}
	}

	.logo-banner {
		padding: 3rem 1rem 3.75rem 1rem;

		.overlay {
			bottom: 3.5rem;
		}
	}

	.section-header {
		.section-tagline,
		.section-description {
			font-size: 1.5rem;
		}
	}

	.main-header {
		font-size: 4rem;
	}
}

@media screen and (min-width: 1024px) {
	.landing-hero {
		h2 {
			font-size: 1.625rem;
		}

		margin-top: -5rem;
		padding: 11.25rem 1rem 12rem;
	}

	.users-section-outer {
		.users-section {
			.feature-blob {
				.blob-text {
					h3 {
						font-size: 4rem;
					}

					p {
						font-size: 1.625rem;
					}
				}
			}
		}
	}

	.logo-banner {
		padding: 4rem 1rem 6.75rem 1rem;

		.overlay {
			bottom: 5rem;
		}
	}

	.section-header {
		.section-tagline,
		.section-description {
			font-size: 1.625rem;
		}
	}

	.main-header {
		font-size: 5.25rem;
	}
}
</style>
