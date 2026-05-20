# Übung: Docker Compose verstehen

**Level:** 1 – Anfänger

## Aufgabe
Lies die `docker-compose.yml` und beantworte die Fragen.

## Fragen

1. Wie viele Services werden definiert?
2. Welche Ports werden gemappt?
3. Was passiert mit `./DB/createTables.sql`?
4. Welche Netzwerk-Konfiguration hat der Backend-Container?

## Lösung

1. 3 Services: postgres, backend, frontend
2. postgres:5432 (intern), frontend:8080:80
3. Wird in `/docker-entrypoint-initdb.d/` gemountet → PostgreSQL führt es beim ersten Start aus
4. `network_mode: host` → teilt sich das Host-Netzwerk
