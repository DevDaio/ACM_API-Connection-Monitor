# Übung: Monitoring-Logik optimieren

**Level:** 3 – Fortgeschritten

## Aufgabe
Der Monitoring-Loop hat ein Problem: Wenn viele Endpoints existieren, kann ein einzelner Durchlauf länger als 5 Sekunden dauern. Optimieren.

## Aktuelle Probleme
1. Sequentielles Prüfen aller Endpoints (kann blockieren)
2. Kein Timeout pro Endpoint
3. DB-Fehler → 10s Pause (auch wenn nur ein Endpoint fehlschlägt)

## Optimierungsvorschläge
1. **Parallele Prüfung** mit `tokio::join_all()` oder `futures::future::join_all()`
2. **Timeout** pro Request via `tokio::time::timeout()`
3. **Fehler-Isolation**: Ein fehlschlagender Endpoint blockiert nicht den Rest

## Code-Grundgerüst
```rust
let handles: Vec<_> = endpoints.iter().map(|ep| {
    let client = &client;
    tokio::spawn(async move {
        // TODO: parallele Prüfung + Timeout
    })
}).collect();

for handle in handles {
    handle.await.unwrap();
}
```

## Bonus
Füge Metriken hinzu: Anzahl erfolgreicher/fehlgeschlagener Checks pro Minute.
