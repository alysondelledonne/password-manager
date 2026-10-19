import { defineConfig } from "vitest/config";
import type { Plugin } from "vite";
import react from "@vitejs/plugin-react";

// Content Security Policy for the built site. Everything is served from the
// site itself: no external scripts, styles, fonts or network calls.
// 'wasm-unsafe-eval' is required to run WebAssembly; it does not allow eval().
const CSP = [
  "default-src 'self'",
  "script-src 'self' 'wasm-unsafe-eval'",
  "style-src 'self'",
  "font-src 'self'",
  "img-src 'self' data:",
  "connect-src 'self'",
  "object-src 'none'",
  "base-uri 'none'",
  "form-action 'none'",
].join("; ");

// Added only to production builds: the dev server needs inline scripts for hot reload.
function cspMetaTag(): Plugin {
  return {
    name: "csp-meta-tag",
    apply: "build",
    transformIndexHtml(html) {
      return html.replace(
        '<meta charset="UTF-8" />',
        `<meta charset="UTF-8" />\n    <meta http-equiv="Content-Security-Policy" content="${CSP}" />`,
      );
    },
  };
}

export default defineConfig({
  // Relative paths, so the site works from any folder (e.g. GitHub Pages).
  base: "./",
  plugins: [react(), cspMetaTag()],
  build: {
    target: "es2022",
    // Never inline assets as data: URLs, so the strict font-src/img-src policy holds.
    assetsInlineLimit: 0,
  },
  test: {
    environment: "jsdom",
    setupFiles: ["./src/test-setup.ts"],
  },
});
