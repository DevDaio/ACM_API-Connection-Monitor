// ─── Vite-Konfiguration ───
import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'    // React JSX-Transform + Fast-Refresh
import tailwindcss from '@tailwindcss/vite'  // Tailwind CSS 4 Integration

export default defineConfig({
  plugins: [react(), tailwindcss()],
  server: {
    port: 8080,           // Dev-Server läuft auf Port 8080 (nicht 5173)
    host: '0.0.0.0',      // Erreichbar unter allen Netzwerk-Interfaces
    proxy: {
      // API-Proxy: leitet alle /acm-Anfragen an das Backend (localhost:3000) weiter
      // Dadurch entfallen CORS-Probleme während der Entwicklung
      '/acm': 'http://localhost:3000',
    },
  },
})
