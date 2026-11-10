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

## Cloudflare releases

The frontend is a Nuxt SSR application deployed directly as Cloudflare Workers from `apps/frontend/wrangler.jsonc`. The `dev` GitHub branch deploys the `dev` Worker to `https://dev.shroudedit.com`. The `prod` branch deploys the `frontend` Worker to `https://shroudedit.com`. A merged version-branch pull request starts only the matching channel workflow. Both workflows use the shared build and deploy steps in `.github/workflows/frontend-deploy.yml`.

The deploy builds the Nuxt application, then Wrangler publishes the generated `.output/server` Worker code and `.output/public` assets. It does not create or upload a ZIP file for Cloudflare. Repository Actions secrets `CF_API_TOKEN` and `CF_ACCOUNT_ID` authenticate the deployment. `SENTRY_AUTH_TOKEN` is used for the release and source map upload.

`.github/workflows/frontend-docker.yml` is a separate publication path. It builds a Node-based container image from the same frontend output and publishes versioned and immutable tags to GitHub Container Registry. It also updates the `dev-latest` or `latest` alias. This GHCR image publication is independent from the Cloudflare Worker deployment.

The dev and production Workers have separate names, domains and build-time variables in `wrangler.jsonc`. Dev uses `api.dev.shroudedit.com`, `archon.dev.shroudedit.com` and `shared-instances.dev.shroudedit.com`. Production uses the corresponding production services. Each release also purges only its own domain cache.
