import "./assets/main.css";

import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import App from "./App";
import { I18nProvider } from "./components/I18nProvider";

// Crash logger — use addEventListener so we don't overwrite prior handlers
window.addEventListener("error", (event) => {
  const { message, filename, lineno, colno, error } = event;
  document.body.innerHTML = `<pre style="color:red;padding:20px;font-family:monospace">CRASH: ${message}\n${filename}:${lineno}:${colno}\n${error?.stack || ""}</pre>`;
});

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
