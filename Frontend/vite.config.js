// ─── Vite-Konfiguration ───
// Host/Port und API-Proxy werden aus .env gelesen (oder Defaults).
const port = parseInt(process.env.FRONTEND_PORT || '8080', 10);
const proxyTarget = process.env.API_PROXY_TARGET || 'http://localhost:3000';

import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'    // React JSX-Transform + Fast-Refresh
import tailwindcss from '@tailwindcss/vite'  // Tailwind CSS 4 Integration

export default defineConfig({
  plugins: [react(), tailwindcss()],
  server: {
    port,
    host: '0.0.0.0',
    proxy: {
      '/acm': proxyTarget,
    },
  },
})
