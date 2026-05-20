# Übung: SQL Query schreiben

**Level:** 2 – Mittel

## Aufgabe
Schreibe eine SQL-Query, die folgende Informationen pro User zurückgibt:
- UserID, Email
- Anzahl der Endpoints
- Anzahl der Log-Einträge (alle Endpoints zusammen)

## Tabelle
- `"user"`: userid, emailadress
- `userendpoint`: userid, endpointid
- `log`: endpointid, status

## Erwartetes Ergebnis
```
userid | emailadress       | endpoint_count | log_count
1      | test@example.com  | 3              | 150
2      | admin@example.com | 1              | 30
```

## Lösung
```sql
SELECT
    u.userid,
    u.emailadress,
    COUNT(DISTINCT ue.endpointid) AS endpoint_count,
    COUNT(l.endpointid) AS log_count
FROM "user" u
LEFT JOIN userendpoint ue ON ue.userid = u.userid
LEFT JOIN log l ON l.endpointid = ue.endpointid
GROUP BY u.userid, u.emailadress
ORDER BY u.userid
```
