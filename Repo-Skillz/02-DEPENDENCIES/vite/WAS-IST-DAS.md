# Vite 8

**Was macht es?** Next-Gen Frontend-Build-Tool. Extrem schneller Dev-Server (ESM HMR), optimierte Production Builds.

**Warum?** Standard für React-Projekte. Schneller als Webpack/CRA.

**Wo?** `Frontend/ACM_Frontend/vite.config.js`

**Konfiguration:**
```js
export default defineConfig({
    plugins: [react(), tailwindcss()],
    server: {
        port: 8080,
        proxy: { '/acm': 'http://localhost:3000' },  // API-Proxy
    },
});
```

**API-Proxy:** In Dev leitet Vite `/acm`-Requests an `localhost:3000` weiter (CORS-Problem umgangen).

**Alternativen:** Webpack (langsamer), Parcel (weniger Features), Turbopack (noch Beta)

**Mini-Tutorial:**
```bash
npm create vite@latest my-app -- --template react
cd my-app && npm install && npm run dev
```
