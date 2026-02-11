# 🚀 Wrokspace Project

A high-performance, hybrid web ecosystem. This project leverages **Blogger** for SEO-driven content on the root domain and a **Salvo (Rust)** powered **HTTP/3** backend to serve a **Svelte CSR** dashboard on the `dash.` subdomain.

## 🏗️ Architecture System

The project is split into three primary layers to balance SEO, performance, and developer experience.

| Layer | Technology | Role | Domain | 
| ----- | ----- | ----- | ----- | 
| **Public Front** | Google Blogger | Content & SEO | `www.antinna.in` | 
| **Application** | Svelte (CSR) | User Dashboard | `dash.antinna.in` | 
| **Infrastructure** | Salvo (Rust) | HTTP/3 API & Static Host | `dash.antinna.in` | 

## 📂 Project Structure

```text
wrokspace/
├── backend/                # Salvo Rust Application
│   ├── src/                # API Handlers, Routes & Middleware
│   │   ├── main.rs         # Entry point & HTTP/3 Logic
│   │   ├── routes/         # API Route Modules
│   │   └── middleware/     # Auth & CORS Logic
│   ├── public/             # Static Assets (Build output from Svelte)
│   ├── certs/              # SSL Certificates (Required for QUIC/HTTP3)
│   └── Cargo.toml          # Rust dependencies (Salvo, Quinn, Serde)
├── frontend/               # Svelte + Vite (CSR Only)
│   ├── src/                # Svelte components and logic
│   ├── public/             # Static public assets
│   ├── vite.config.js      # Build pipeline configuration
│   └── svelte.config.js    # Svelte preprocessor settings
├── blogger/                # Root Domain Configuration
│   └── indie.xml           # Custom Blogger Theme Template
└── README.md               # Project documentation
```
## 🛠️ Technology Breakdown

### 1. Frontend: Svelte (Vite)
* **Mode:** 100% Client-Side Rendered (CSR).
* **Pre-processing:** Uses `vitePreprocess` for optimized styling and scripting.
* **Integration:** Configured via `vite.config.js` to build directly into `../backend/public`.

### 2. Backend: Salvo (Rust)
* **Protocol:** Implements **HTTP/3 (QUIC)** via `QuinnListener` alongside standard TCP.
* **Static Hosting:** Efficiently serves the Svelte app using `StaticDir`.
* **SPA Fallback:** Implements a "catch-all" handler to ensure that browser refreshes on Svelte routes (e.g., `/settings`) return the main `index.html`.

### 3. Blogger: Indie Theme
* **Theme:** A custom XML-based theme (`indie.xml`) optimized for performance.
* **Hosting:** Managed by Google, providing high availability for the landing page and blog.

## 🚀 Setup & Development

### Prerequisite: SSL Certificates
HTTP/3 requires valid TLS. For local development, place your `cert.pem` and `key.pem` in `backend/certs/`.

### Step 1: Build the Frontend
Navigate to the frontend directory and install dependencies.

\```bash
cd frontend
npm install
npm run build
\```

*The build output is automatically moved to `backend/public` via Vite configuration.*

### Step 2: Run the Salvo Server
\```bash
cd ../backend
cargo run --release
\```

The server will now be listening on `0.0.0.0:443` (UDP/TCP).

### Step 3: Blogger Deployment
1. Open your Blogger Dashboard.
2. Navigate to **Theme** > **Edit HTML**.
3. Copy the contents of `blogger/indie.xml` and paste it into the editor.

## ⚙️ Key Configurations

### Vite Integration (`frontend/vite.config.js`)
```javascript
import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';

export default defineConfig({
  plugins: [svelte()],
  build: {
    outDir: '../backend/public',
    emptyOutDir: true
  }
});
```

### Salvo SPA Routing (`backend/src/main.rs`)
```rust
Router::with_path("<**path>")
    .get(StaticDir::new(["public"])
    .defaults("index.html")
    .with_fallback("public/index.html"))
```

## 📝 Troubleshooting & FAQ

* **Q: I get a 404 when refreshing the dashboard page.**
  * **A:** Ensure `with_fallback("public/index.html")` is correctly set in your Salvo router. This ensures the server redirects unknown paths to the Svelte entry point.
* **Q: HTTP/3 is not working in the browser.**
  * **A:** HTTP/3 requires a valid HTTPS connection. Ensure your firewall allows **UDP** traffic on port 443, not just TCP. (For me this is Just for Mobile App).In this architecture, HTTP/3 (QUIC) is optimized specifically for the Mobile App client to ensure stable API performance over unstable networks. Browsers will automatically fall back to standard HTTPS (TCP) if UDP port 443 is blocked or if the browser's QUIC implementation differs.
* **Q: Svelte changes are not reflecting.**
  * **A:** You must run `npm run build` inside the `frontend` folder for changes to be moved into the Salvo `public` directory.

## 📝 License
This project is part of the **Wrokspace** ecosystem. All rights reserved.
