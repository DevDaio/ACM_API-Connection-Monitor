# React 19

**Was macht es?** Deklarative UI-Library für interaktive Web-Oberflächen.

**Warum?** Komponenten-basiert, riesiges Ökosystem, Hooks für State/Lifecycle.

**Wo?** `Frontend/ACM_Frontend/src/` — alle .jsx-Dateien

**Verwendete Konzepte:**
- `useState` — lokaler State (user, endpoints, modal-open)
- `useEffect` — Side-Effekte (Polling, localStorage, Theme)
- `useRef` — Mutable Referenz (Modal-Check für Polling-Pause)
- `useContext` — Globaler Theme-Context
- `useMemo` — gefilterte Log-Einträge
- Props — Datenfluss von App → Child-Komponenten

**Alternativen:** Vue (leichter), Svelte (kein VDOM), Solid (feingranular)

**Mini-Tutorial:**
```jsx
import { useState } from 'react';

function Counter() {
    const [count, setCount] = useState(0);
    return <button onClick={() => setCount(c => c + 1)}>{count}</button>;
}
```
