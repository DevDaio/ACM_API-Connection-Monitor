# Übung: Eigenes Modal bauen

**Level:** 2 – Mittel

## Aufgabe
Baue ein neues Modal `ApiInfoModal.jsx`, das folgende Infos anzeigt:
- Backend-Version (hardcoded: "0.1.0")
- Anzahl der aktiven Endpoints
- Aktuelles Theme

## Anforderungen
- Nutze die existierende `Modal.jsx`-Komponente
- Erhalte `endpoints` und `theme` als Props
- Zeige einen Close-Button
- Integriere es in App.jsx

## Struktur
```jsx
function ApiInfoModal({ isOpen, onClose, endpoints, theme }) {
    // TODO
}
```

## Bonus
Füge einen Refresh-Button hinzu, der die Daten neu lädt.
