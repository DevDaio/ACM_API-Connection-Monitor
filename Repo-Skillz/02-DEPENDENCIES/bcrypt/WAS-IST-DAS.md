# bcrypt 0.16

**Was macht es?** Passwort-Hashing mit dem bcrypt-Algorithmus.

**Warum?** User-Passwörter sicher speichern. Niemals Klartext in der DB!

**Wo?** `Backend/API/src/main.rs` — Zeilen 118-123 (hash), 159-165 (verify)

**Wie?**
```rust
let hashed = bcrypt::hash(&password, bcrypt::DEFAULT_COST)?;
let valid = bcrypt::verify(&password, &hashed)?;
```

**DEFAULT_COST = 12** (Anzahl der Runden = 2^12). Höher = sicherer, aber langsamer.

**Alternativen:** argon2 (noch sicherer), pbkdf2 (älter)

**Mini-Tutorial:**
```rust
let hash = bcrypt::hash("mein_passwort", 10).unwrap();
assert!(bcrypt::verify("mein_passwort", &hash).unwrap());
```
