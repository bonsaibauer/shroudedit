# ![Frontend Cover](/.github/assets/web_cover.png)

# ShroudEdit Web Interface

Welcome to the ShroudEdit Web Interface! This web-based platform is designed for discovering, downloading, and managing Enshrouded mods.

If you're not a developer and you've stumbled upon this repository, you can access the web interface on the [ShroudEdit website](https://shroudedit.com).

## Development

### Pre-requisites

Before you begin, ensure you have the following installed on your machine:

- [Node.js](https://nodejs.org/en/)
- [pnpm](https://pnpm.io/)

### Setup

Follow these steps to set up your development environment:

```bash
pnpm install
pnpm web:dev
```

You should now have a development build of the web interface running with hot-reloading enabled. Any changes you make to the code will automatically refresh the browser.

## Cloudflare deployment

Run **Deploy frontend** from GitHub Actions and choose a version branch in `X.Y.Z` format to deploy `stable` manually. Every push to a branch matching `*.*.*` deploys `dev`, including pushes created by merging a pull request. Pull requests themselves run through regular CI and do not deploy. Each deployment validates the branch version, writes it to the root `VERSION` file for that build, and checks that its source branch has not moved during the build. Update the checked-in `VERSION` file when preparing a new version branch. Local and deployed frontend builds expose it as `public.version` in Nuxt runtime config.

The `stable` channel uses the default Worker configuration for `https://shroudedit.com`. The `dev` channel uses the Wrangler `dev` environment for `https://dev.shroudedit.com`. Each channel injects its own build variables and purges only its own domain cache. The deploy builds the Nuxt application and publishes `.output/server` and `.output/public` directly through Wrangler.

Repository Actions secrets `CF_API_TOKEN` and `CF_ACCOUNT_ID` authenticate the deployment. `SENTRY_AUTH_TOKEN` is used to upload source maps and create the Sentry release. Both Worker configurations remain in `apps/frontend/wrangler.jsonc`.
