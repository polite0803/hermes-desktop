import "./assets/main.css";

import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import App from "./App";
import { I18nProvider } from "./components/I18nProvider";

// Crash logger
window.onerror = (msg, src, line, col, err) => {
  document.body.innerHTML = `<pre style="color:red;padding:20px;font-family:monospace">CRASH: ${msg}\n${src}:${line}:${col}\n${err?.stack || ""}</pre>`;
};

try {
  createRoot(document.getElementById("root")!).render(
    <StrictMode>
      <I18nProvider>
        <App />
      </I18nProvider>
    </StrictMode>,
  );
} catch (e) {
  document.body.innerHTML = `<pre style="color:red;padding:20px">FATAL: ${e}</pre>`;
}
