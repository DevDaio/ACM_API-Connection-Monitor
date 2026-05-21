// ─── React Entry-Point ───
// StrictMode: Entwicklungshilfe (doppeltes Rendern, deprecated-API-Warnungen)
import { StrictMode } from 'react'
// createRoot: React 19 API zum Mounten in ein DOM-Element
import { createRoot } from 'react-dom/client'
import './index.css'  // Globales CSS
import App from './App.jsx'

// Sucht das <div id="root"> in index.html und mountet React dort
createRoot(document.getElementById('root')).render(
  <StrictMode>
    <App />
  </StrictMode>,
)
