# Anpassungen

## 2026-10-11 — API-Server: Hetzner CX23

Für den ersten API-Server wurde ein Hetzner Cloud CX23 eingerichtet. Die gebuchte Größe umfasst 2 vCPU, 4 GB RAM, 40 GB lokalen Datenträger und laut Bestellung 20 TB ausgehenden Traffic. Der angezeigte Preis beträgt 0,01 € pro Stunde beziehungsweise ungefähr 6,53 € pro Monat.

Der Server ist als einzelner Produktions-Host für die API vorgesehen. Die Staging-Umgebung bleibt auf das Frontend beschränkt; es wird keine zweite API- oder Speicherumgebung angelegt. Veröffentlichte Blueprint-Dateien sollen in den bestehenden Cloudflare-R2-Bucket `shroudedit` geschrieben werden, nicht dauerhaft auf die 40-GB-Systemdisk.

**Ressourcenhinweis:** Der CX23 ist ein kleiner Startserver. Der aktuelle `docker-compose.yml`-Stack startet neben Labrinth unter anderem PostgreSQL, Redis, ClickHouse, Redpanda und drei Elasticsearch-Knoten mit jeweils 512 MB Java-Heap. Dieser vollständige Entwicklungsstack ist für 4 GB RAM nicht als Produktionsdeployment geeignet. Vor dem API-Deployment müssen wir die Produktionsdienste gezielt auswählen beziehungsweise verwaltete Dienste verwenden und Elasticsearch/weitere nicht benötigte Entwicklungsdienste aus dem Startpfad nehmen. Datenbank-, Redis-, Such- oder sonstige Verwaltungsports dürfen nicht öffentlich erreichbar sein.

## 2026-10-11 — Cloudflare Workers Free: CPU-Limit

Die explizite Einstellung `limits.cpu_ms: 5000` wurde aus `apps/frontend/wrangler.jsonc` entfernt. Cloudflare hat den Deploy mit Fehler `100328` abgelehnt, weil der Workers-Free-Plan keine benutzerdefinierte CPU-Grenze unterstützt.

Der Worker verwendet damit das Standardlimit des Free-Plans: 10 ms CPU-Zeit pro Anfrage. Das ist reine Rechenzeit; Wartezeit auf Antworten anderer Server zählt nicht dazu. Überschreitet eine Anfrage die Grenze, kann Cloudflare sie zur Laufzeit abbrechen. Der Build selbst war erfolgreich; blockiert wurde die Bereitstellung.

Ein späterer Wechsel zum Workers-Paid-Plan ermöglicht eine höhere CPU-Grenze. Die aktuelle Preis- und Limitübersicht steht in der [Cloudflare-Dokumentation](https://developers.cloudflare.com/workers/platform/pricing/).

## 2026-10-11 — Labrinth-API: Uploadlimit für Cloudflare Free

Cloudflare Free begrenzt den Inhalt eines proxied HTTP-Requests auf 100 MB. Labrinth nimmt Versionsdateien über solche API-Requests entgegen; die bisherigen Limits von 256 MiB für `.sfbp` und 500 MiB für andere Versionsdateien lagen über dieser Grenze und konnten deshalb schon am Cloudflare-Proxy mit `413 Request Entity Too Large` abgewiesen werden.

Die bestehende Größenprüfung für `.sfbp`-, Mod- und Modpack-Dateien wurde auf 95 MiB gesetzt. Cloudflare zählt den vollständigen Request einschließlich Multipart-Formulardaten. Die vorhandene Labrinth-Prüfung gilt pro Datei; kleinere Speziallimits, zum Beispiel für Galeriebilder, bleiben bestehen. Details zum [Cloudflare-Uploadlimit](https://developers.cloudflare.com/support/troubleshooting/http-status-codes/4xx-client-error/error-413/).
