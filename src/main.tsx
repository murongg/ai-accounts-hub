import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import "./App.css";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <main className="min-h-screen bg-base-200 text-base-content">
      <App />
    </main>
  </React.StrictMode>,
);
