# Dependency-Graph

```mermaid
graph LR
    subgraph RUST["Rust Backend Dependencies"]
        AX[axum 0.8] -->|HTTP Framework| APP
        TOK[tokio 1.52] -->|Async Runtime| AX
        TOK -->|Async| RQ[reqwest]
        TOK -->|Spawn| MON
        SQ[sqlx 0.8] -->|DB| APP
        BC[bcrypt 0.16] -->|Hashing| APP
        SE[serde 1.0] -->|Serde| APP
        SJ[serde_json 1.0] -->|JSON| APP
        CH[chrono 0.4] -->|Dates| SQ
        TH[tower-http 0.6] -->|CORS| AX
        TW[tower 0.5] -->|Middleware| AX
        ENV[dotenv 0.15] -->|ENV| APP
    end

    subgraph REACT["React Frontend Dependencies"]
        RE[react 19] -->|UI| FE
        RD[react-dom 19] -->|DOM| RE
        TW2[tailwindcss 4] -->|CSS| FE
        VT[vite 8] -->|Build| FE
    end

    subgraph INFRA["Infrastructure"]
        PG[postgres 17]
        NGINX[nginx:alpine]
    end

    APP[ACM API] -->|CORS| FE[ACM Frontend]
    SQ -->|Connection| PG
    APP -->|Docker| NGINX
    FE -->|Docker| NGINX
```

## Production vs Dev Dependencies

### Backend (Rust) — Production
| Crate | Version | Zweck |
|-------|---------|-------|
| axum | 0.8.9 | Async HTTP framework |
| tokio | 1.52.3 | Async runtime |
| serde | 1.0.228 | Serialisierung |
| serde_json | 1.0.149 | JSON-Handling |
| sqlx | 0.8 | PostgreSQL-Treiber |
| bcrypt | 0.16 | Passwort-Hashing |
| tower-http | 0.6 | CORS-Middleware |
| chrono | 0.4 | Datum/Zeit-Typen |
| reqwest | 0.12 | HTTP-Client für Monitoring |
| dotenv | 0.15 | .env-Laden |
| tower | 0.5 | Middleware-Layer |

### Frontend (React) — Production
| Package | Version | Zweck |
|---------|---------|-------|
| react | ^19.2.6 | UI-Library |
| react-dom | ^19.2.6 | DOM-Rendering |
| tailwindcss | ^4.3.0 | Utility-CSS |
| @tailwindcss/vite | ^4.3.0 | Tailwind Vite-Plugin |

### Frontend — Dev
| Package | Version | Zweck |
|---------|---------|-------|
| vite | ^8.0.12 | Build-Tool |
| @vitejs/plugin-react | ^6.0.1 | React-Integration |
| eslint | ^10.3.0 | Linter |
