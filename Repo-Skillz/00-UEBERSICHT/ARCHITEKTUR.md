# Architektur

```mermaid
graph TB
    subgraph Frontend["Frontend (React 19 + Vite)"]
        LP[LandingPage] -->|Login/Register| API
        DB[Dashboard] -->|CRUD Endpoints| API
        EC[EndpointCard] -->|Status| SL[Sparkline]
        MOD[Modal-System] -->|CRUD| API
        TC[ThemeSwitcher] -->|data-theme| CSS
    end

    subgraph API["Backend (Rust + Axum)"]
        ROUTER[Router /acm/*] --> HANDLER{Handler}
        HANDLER -->|Queries| DB_LAYER[async_services]
        DB_LAYER -->|sqlx| PG[(PostgreSQL)]
        MON[Monitoring Loop] -->|reqwest GET| ENDPOINTS[API-Endpunkte]
        MON -->|insert_log| DB_LAYER
    end

    subgraph INFRA["Infrastruktur"]
        NGINX[Nginx Reverse Proxy] -->|/acm| API
        NGINX -->|/*| VITE[Vite Dev / Static]
        DOCKER[Docker Compose]
    end

    API --> DOCKER
    PG --> DOCKER
    NGINX --> DOCKER
```

## Datenfluss

```
User → Browser → Nginx (:80) → /acm → Axum Backend (:3000) → sqlx → PostgreSQL
                                    → /*  → Static Files (index.html + JS)
```

## Monitoring-Loop

```
Tokio-Spawn → run_monitoring_loop()
  └─ loop (alle 5s)
       └─ get_endpoints_with_intervals()
       └─ für jeden Endpoint:
            ├─ last_checked prüfen (Intervall eingehalten?)
            ├─ reqwest GET → Status (bool)
            └─ insert_log (endpointid, status)
```

## DB-Schema

```
user (userid PK, emailadress, password)
endpoint (endpointid PK, url)
userendpoint (userid FK, endpointid FK) → M:N
intervall (endpointid PK FK, seconds)
log (endpointid FK, status, statusdate, statustime)
```
