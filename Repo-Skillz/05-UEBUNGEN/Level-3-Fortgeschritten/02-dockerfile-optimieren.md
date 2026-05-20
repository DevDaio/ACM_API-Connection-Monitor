# Übung: Dockerfile optimieren

**Level:** 3 – Fortgeschritten

## Aufgabe
Das aktuelle Backend-Dockerfile baut jedes Mal komplett neu. Optimiere es.

## Aktuelles Dockerfile
```dockerfile
FROM rust:latest AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release
```

## Problem
- Jede Code-Änderung erfordert vollständigen Rebuild
- Kein Cargo-Caching (Abhängigkeiten werden nicht separat gecached)
- `rust:latest` ist ~1.5GB

## Optimierungen
1. **Cargo-Cache**: COPY Cargo.* → cargo build (dummy) → COPY src → cargo build
2. **Slim-Image**: `rust:slim-bookworm` statt `rust:latest`
3. **Target-Verzeichnis**: Nutze --mount type=cache für Docker BuildKit

## Optimiertes Dockerfile (Templat)
```dockerfile
# TODO: Optimiere das Dockerfile
```
