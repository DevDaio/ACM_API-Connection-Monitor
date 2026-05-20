# Cheatsheet

## Rust / Axum

```rust
// Router
Router::new().route("/path", get(handler)).layer(cors).with_state(state);

// Handler-Signatur
async fn handler(State(state): State<Arc<AppState>>) -> Result<Json<T>, (StatusCode, Json<Error>)>

// Shared State
Arc::new(AppState { pool });

// CORS
CorsLayer::new().allow_origin(Any).allow_methods([Method::GET, Method::POST]).allow_headers(Any);
```

## sqlx

```rust
// Query → Struct
sqlx::query_as::<_, T>("SELECT * FROM table WHERE id = $1").bind(id).fetch_one(&pool).await?;

// Execute (kein Return)
sqlx::query("UPDATE table SET col = $1").bind(val).execute(&pool).await?;

// Optional
sqlx::query_as::<_, T>("SELECT * FROM table WHERE email = $1").bind(email).fetch_optional(&pool).await?;

// Upsert
sqlx::query("INSERT INTO ... VALUES ($1, $2) ON CONFLICT (id) DO UPDATE SET col = EXCLUDED.col")
```

## React Hooks

```jsx
// State
const [state, setState] = useState(initial);

// Effect
useEffect(() => { /* side effect */ return () => /* cleanup */ }, [deps]);

// Ref
const ref = useRef(initial);
// ref.current ändert ohne Re-Render

// Context
const { value } = useContext(MyContext);
```

## Docker

```bash
# Bauen
docker build -t acm-backend -f Backend/API/Dockerfile.local Backend/API/
docker build -t acm-frontend Frontend/ACM_Frontend/

# Starten
docker compose up -d

# Stoppen
docker compose down

# Logs
docker compose logs -f backend
```

## PostgreSQL

```sql
-- Tabellen anzeigen
\dt

-- Beschreibung
\d "user"

-- Query
SELECT * FROM "user" JOIN userendpoint USING (userid);
```
