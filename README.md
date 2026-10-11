# ShroudEdit

[![Made with love](https://img.shields.io/badge/Made%20with-%E2%9D%A4%EF%B8%8F-c78aff?style=flat)](https://github.com/ShroudEdit)
[![Repository](https://img.shields.io/badge/Repository-ShroudEdit-blue?style=flat&logo=github)](https://github.com/ShroudEdit/code)
[![Website](https://img.shields.io/badge/Website-shroudedit.com-c78aff?style=flat)](https://shroudedit.com)
[![Report a bug](https://img.shields.io/badge/Report-Bug-critical?style=flat&logo=github)](https://github.com/ShroudEdit/code/issues/new)
[![Request a feature](https://img.shields.io/badge/Request-Feature-green?style=flat&logo=github)](https://github.com/ShroudEdit/code/issues/new)
[![Issues](https://img.shields.io/github/issues/ShroudEdit/code?style=flat&label=Issues)](https://github.com/ShroudEdit/code/issues)
[![Pull requests](https://img.shields.io/github/issues-pr/ShroudEdit/code?style=flat&label=Pull%20requests)](https://github.com/ShroudEdit/code/pulls)
[![Contributors](https://img.shields.io/github/contributors/ShroudEdit/code?style=flat)](https://github.com/ShroudEdit/code/graphs/contributors)

![GitHub stars](https://img.shields.io/github/stars/ShroudEdit/code?style=social)
![GitHub forks](https://img.shields.io/github/forks/ShroudEdit/code?style=social)

## Welcome!

ShroudEdit is a website and API for discovering and publishing Enshrouded community projects. This monorepo contains the frontend, Labrinth API, shared UI, and API client.

The ShroudEdit API accepts ShroudForge Blueprint (`.sfbp`) V7 files and stores them unchanged. The format and OAuth S256 PKCE upload flow for connected editor clients are described in the [SFBP reference](apps/labrinth/SFBP_FORMAT.md).

## Use ShroudEdit

- [Open the website](https://shroudedit.com)
- [Browse Enshrouded mods](https://shroudedit.com/discover/mods)
- [Browse World Editor blueprints](https://shroudedit.com/discover/schematics)
- [Read the documentation](https://docs.shroudedit.com)

## Local development

This repository uses pnpm workspaces and Turborepo.

1. Install the repository's pnpm version and dependencies with `pnpm install`.
2. Configure the frontend environment for the services you use locally.
3. Start the website with `pnpm web:dev`.

Read the root [AGENTS.md](AGENTS.md) and the project-level instructions before changing a project. [Frontend setup](apps/frontend/AGENTS.md) covers website development; [Labrinth setup](apps/labrinth/AGENTS.md) covers API and database requirements.

## Repository layout

| Path | Purpose |
| --- | --- |
| `apps/frontend` | ShroudEdit website |
| `apps/labrinth` | Rust API, accounts, projects, and version uploads |
| `packages/api-client` | Shared API client |
| `packages/ui` | Shared Vue components |
| `packages/assets` | Shared icons, illustrations, and styles |

## Monetization

Monetization features are disabled by default. Set `MONETIZATION_ENABLED=true` for both the website and API to enable them again. The switch keeps the implementation and existing project data available while hiding the related routes and stopping their background jobs.

## Contributing

Before opening a pull request, read the applicable project instructions and [contributing guide](https://docs.shroudedit.com/contributing/getting-started/). Report bugs or request features through [GitHub Issues](https://github.com/ShroudEdit/code/issues).

## Security

Report security issues through the [ShroudEdit security page](https://shroudedit.com/legal/security).

## License

Apps and packages may use different licenses. Check the license file for the component you use, such as [frontend](apps/frontend/LICENSE), [Labrinth](apps/labrinth/LICENSE.txt), [API client](packages/api-client/LICENSE), [UI](packages/ui/LICENSE), or [assets](packages/assets/LICENSE).
