# Konzept: React-Komponenten und State-Management

## Was ist das?
React ist eine deklarative UI-Library. In diesem Projekt nutzen wir funktionale Komponenten mit Hooks.

## Verwendete Hooks

### useState
```jsx
const [user, setUser] = useState(null);
const [endpoints, setEndpoints] = useState([]);
```

### useEffect
```jsx
// Polling alle 10s (außer bei offenem Modal)
useEffect(() => {
    const poll = setInterval(() => {
        if (anyModalRef.current) return;
        api.getHome(user.userid).then(d => setEndpoints(mapEndpoints(d)));
    }, 10000);
    return () => clearInterval(poll);
}, [user]);
```

### useRef
```jsx
const anyModalRef = useRef(anyModalOpen);
useEffect(() => { anyModalRef.current = anyModalOpen; }, [anyModalOpen]);
```

### useContext (ThemeContext)
```jsx
const { theme, setTheme, themes, current } = useTheme();
```

## Komponenten-Hierarchie

```
App (State-Hub)
├── LandingPage (Login)
├── Dashboard (Main View)
│   ├── ThemeSwitcher
│   └── EndpointCard[] (Tabelle)
│       └── Sparkline (Mini-Chart)
├── CreateAccountModal
├── AddEndpointModal
├── SetIntervallModal
├── DeleteConfirmModal
├── AccountSettingsModal
├── LogModal
└── EditUrlModal
```

## Design-Pattern: State-Hub (App.jsx)

Alle Modals und die Dashboard-Logik werden über Props gesteuert. `App.jsx` ist die zentrale State-Verwaltung:
- `user`-State (Login/Logout)
- `endpoints`-Array (Monitor-Daten)
- Modal-open-Booleans
- Callback-Funktionen als Props

## Warum dieses Pattern?
- Einfach und überschaubar (kein Router, kein Redux)
- Prop-Drilling ist hier akzeptabel (flache Hierarchie)
- Theme-Context für globale Styles

## Übungen
- 05-UEBUNGEN/Level-1-Anfaenger/02-react-hooks-erkennen.md
- 05-UEBUNGEN/Level-2-Mittel/02-eigenes-modal-bauen.md
