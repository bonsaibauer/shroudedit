# Anpassungen

## 2026-10-11 — API-Deployment

Die API-Pipeline veröffentlicht getrennte Images für `stable` und `dev`. Dieses Repository enthält den lokalen Docker-Compose-Stack, aber keine Zielserver- oder Runtime-Deploy-Konfiguration. Der aktuelle Server und seine Kapazität werden hier nicht festgelegt.

**Ressourcenhinweis:** Der lokale `docker-compose.yml`-Stack startet neben Labrinth unter anderem PostgreSQL, Redis, ClickHouse, Redpanda und drei Elasticsearch-Knoten mit jeweils 512 MB Java-Heap. Vor einem Server-Deploy müssen die tatsächlich benötigten Dienste und deren Kapazität festgelegt werden. Datenbank-, Redis-, Such- oder sonstige Verwaltungsports dürfen nicht öffentlich erreichbar sein.

**ClickHouse-Konfiguration für Production noch offen:** Labrinth verwendet ohne explizite Umgebungsvariable den Datenbanknamen `staging_ariadne` (`apps/labrinth/src/env.rs`); auch die lokalen Compose-Umgebungen setzen diesen Namen. Die TTL-Sonderbehandlung in `apps/labrinth/src/clickhouse/mod.rs` prüft dagegen auf `staging_analytics`. Vor dem Produktivbetrieb muss die Production-Datenbank ausdrücklich festgelegt und die TTL-Regel mit diesem Namen abgeglichen werden. Der Server darf nicht versehentlich den Staging-Datenbanknamen als Production-Ziel übernehmen.

## 2026-11-10 — Labrinth-API-Deployment per Channel

Pull Requests bauen und prüfen die Labrinth-API, veröffentlichen aber kein Image. Jeder Push mit API-relevanten Änderungen auf einen Versions-Branch nach dem Muster `*.*.*` veröffentlicht automatisch das `dev`-Image, auch ein Push durch das Mergen eines Pull Requests. Die Version wird wie in ShroudForge aus dem Namen des ausgelösten Branches abgeleitet und für den Build in die Root-Datei `VERSION` geschrieben. Das API-Image trägt diese Version als OCI-Metadatum. `stable` wird manuell über `Labrinth Build and Publish` in GitHub Actions von einem Versions-Branch veröffentlicht. Vor der Aktualisierung der Image-Aliase prüft der Workflow, ob der Quellbranch noch auf demselben Commit steht. `stable` aktualisiert die Aliase `stable` und `latest`. `dev` aktualisiert `dev` und `dev-latest`.

**Backend-Runtime-Deploy:** Die API-Workflows veröffentlichen getrennte Images, aber dieses Repository enthält keine Server-Compose-Datei oder Runtime-Deploy-Schritte, die diese Images starten. Der tatsächliche Hostingpfad wird außerhalb dieser Repositorykonfiguration festgelegt.

## 2026-11-10 — Frontend-Deployment per Channel

Der Channel `stable` wird manuell über den GitHub-Actions-Workflow `Deploy frontend` von einem Versions-Branch veröffentlicht. Jeder Push auf einen Versions-Branch nach dem Muster `*.*.*` deployt automatisch `dev`, auch ein Push durch das Mergen eines Pull Requests. PRs selbst laufen durch die normale CI und deployen nicht. Der Workflow übernimmt die Versionsnummer aus dem Branch-Namen und schreibt sie für den Build in die Root-Datei `VERSION`. Vor dem Deploy prüft er, ob der Quellbranch noch auf dem erwarteten Commit steht. `stable` verwendet die Standardkonfiguration in `apps/frontend/wrangler.jsonc` für `shroudedit.com`. `dev` verwendet die Wrangler-Umgebung `dev` für `dev.shroudedit.com`. Jeder Lauf leert nur den Cache der eigenen Domain. Benötigt werden die GitHub-Secrets `CF_API_TOKEN`, `CF_ACCOUNT_ID` und `SENTRY_AUTH_TOKEN`.

Die frühere eigenständige GHCR-Containerveröffentlichung wurde entfernt. Frontend-Deployments laufen nur über GitHub Actions und Wrangler. Das lokale `cf-dev`-Skript startet weiterhin einen Entwicklungsserver und veröffentlicht kein Deployment.

## 2026-10-11 — ShroudEdit-Desktop-App-Build und Release entfernt

ShroudEdit entwickelt und veröffentlicht keine eigene Desktop-Fenster-App. Im Repository gab es keine `apps/app`- oder `apps/app-frontend`-Quellverzeichnisse mehr, aber die separaten GitHub-Actions `theseus-build.yml` und `theseus-release.yml` waren noch vorhanden und konnten weiterhin Desktop-Installationsdateien bauen, Releases erstellen und Updates in den ShroudEdit-App-Dateispeicher `launcher-files.shroudedit.com` hochladen. Beide Workflows wurden entfernt. API- und Website-Funktionen für OAuth, Mod- und Blueprint-Uploads, Downloads sowie externe Launcher bleiben bestehen; dazu zählt auch die geplante Integration mit dem separaten ShroudForge Launcher. Historische Download-Statistiken behalten ihre frühere Theseus-Kennung, damit bereits gespeicherte Auswertungen weiterhin zugeordnet werden.

## 2026-10-12 — Abschlussprüfung der Deploy- und Launcher-Pfade

Beim letzten CI-nahen Production-Frontend-Build antwortete `api.shroudedit.com` mit einer Cloudflare-Challenge. Dadurch schlug das Abrufen des Tag-/Statuszustands fehl und der absichtlich strenge Production-Build brach ab. Vor einer produktiven Frontend-Freigabe muss der Buildzugriff auf diesen API-Endpunkt funktionieren; die Fehlerprüfung sollte nicht entfernt werden, weil sie einen Deploy mit unvollständigem Seitenzustand verhindert.

Die API-Client-Anbindung an `launcher-meta.shroudedit.com` bleibt erhalten. Der nicht funktionsfähige Workflow `.github/workflows/daedalus-docker.yml` wurde entfernt: Die referenzierten Verzeichnisse `apps/daedalus_client` und `packages/daedalus` sowie das Cargo-Paket `daedalus_client` fehlen, und der Workflow hatte widersprüchliche Push- und Deploy-Branches. Die Entfernung betrifft nur den Daedalus-Container-Build und ArgoCD-Deploy. Sie entfernt weder OAuth noch ShroudForge-Mod- oder `.sfbp`-Blueprint-Funktionen noch den veröffentlichten `launcher-meta`-API-Client. Externe Verbraucher dieses API-Client-Moduls wurden nicht geprüft; deshalb bleibt es im Paket.

Der separate Workflow `.github/workflows/api-client-release.yml` veröffentlicht `@shroudedit/api-client` automatisch nach npm, wenn sich das Paket auf dem Standardbranch ändert. Das ist ein Paket-Release für externe API-Clients und kein Website- oder Hetzner-Deploy. Der Workflow bleibt wegen der Launcher-Integration erhalten.

## 2026-10-11 — Cloudflare Workers Free: CPU-Limit

Die explizite Einstellung `limits.cpu_ms: 5000` wurde aus `apps/frontend/wrangler.jsonc` entfernt. Cloudflare hat den Deploy mit Fehler `100328` abgelehnt, weil der Workers-Free-Plan keine benutzerdefinierte CPU-Grenze unterstützt.

Der Worker verwendet damit das Standardlimit des Free-Plans: 10 ms CPU-Zeit pro Anfrage. Das ist reine Rechenzeit; Wartezeit auf Antworten anderer Server zählt nicht dazu. Überschreitet eine Anfrage die Grenze, kann Cloudflare sie zur Laufzeit abbrechen. Der Build selbst war erfolgreich; blockiert wurde die Bereitstellung.

Ein späterer Wechsel zum Workers-Paid-Plan ermöglicht eine höhere CPU-Grenze. Die aktuelle Preis- und Limitübersicht steht in der [Cloudflare-Dokumentation](https://developers.cloudflare.com/workers/platform/pricing/).

## 2026-10-11 — Labrinth-API: Uploadlimit für Cloudflare Free

Cloudflare Free begrenzt den Inhalt eines proxied HTTP-Requests auf 100 MB. Labrinth nimmt Versionsdateien über solche API-Requests entgegen; die bisherigen Limits von 256 MiB für `.sfbp` und 500 MiB für andere Versionsdateien lagen über dieser Grenze und konnten deshalb schon am Cloudflare-Proxy mit `413 Request Entity Too Large` abgewiesen werden.

Die bestehende Größenprüfung für `.sfbp`-, Mod- und Modpack-Dateien wurde auf 95 MiB gesetzt. Cloudflare zählt den vollständigen Request einschließlich Multipart-Formulardaten. Die vorhandene Labrinth-Prüfung gilt pro Datei; kleinere Speziallimits, zum Beispiel für Galeriebilder, bleiben bestehen. Details zum [Cloudflare-Uploadlimit](https://developers.cloudflare.com/support/troubleshooting/http-status-codes/4xx-client-error/error-413/).
