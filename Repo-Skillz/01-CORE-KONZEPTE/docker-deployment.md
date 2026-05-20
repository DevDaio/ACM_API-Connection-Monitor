# Konzept: Docker-Deployment

## Was ist das?
Multi-Container-Setup mit drei Services: PostgreSQL, Rust-Backend, React-Frontend (served via Nginx).

## docker-compose.yml
```yaml
services:
  postgres:
    image: postgres:17
    volumes:
      - pgdata:/var/lib/postgresql/data
      - ./DB/createTables.sql:/docker-entrypoint-initdb.d/01-init.sql

  backend:
    build: ./Backend/API
    network_mode: host
    depends_on: [postgres]

  frontend:
    build: ./Frontend/ACM_Frontend
    ports: ["8080:80"]
```

## Backend Dockerfile (Multi-Stage)
```dockerfile
# Stage 1: Build
FROM rust:latest AS builder
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release

# Stage 2: Distroless (minimal)
FROM gcr.io/distroless/cc-debian12:latest
COPY --from=builder /app/target/release/Backend /usr/local/bin/Backend
```

## Frontend Dockerfile (Nginx)
```dockerfile
FROM nginx:alpine
COPY dist /usr/share/nginx/html
# Nginx-Config mit API-Proxy
location /acm {
    proxy_pass http://172.19.0.1:3000;
}
```

## Deployment-Ablauf
1. `cargo build --release` (Rust)
2. `docker build -t acm-backend -f Dockerfile.local .`
3. `npm ci && npm run build` (React)
4. `docker build -t acm-frontend .`
5. `docker compose up -d`

## Warum Docker?
- Reproduzierbare Umgebungen
- Ein-Klick-Deployment via setup.sh
- Isolierte Services
- Skalierbar

## Übungen
- 05-UEBUNGEN/Level-3-Fortgeschritten/02-dockerfile-optimieren.md
