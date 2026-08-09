import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import App from "./App";
import Website from "./Website";
import { isTauriRuntime } from "./runtime";
import "./styles.css";

const surface = isTauriRuntime() ? "desktop" : "website";
document.documentElement.dataset.surface = surface;

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    {surface === "desktop" ? <App /> : <Website />}
  </StrictMode>,
);
