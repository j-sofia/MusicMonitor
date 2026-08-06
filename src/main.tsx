import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { DisplayApp } from "./DisplayApp";
import { SettingsApp } from "./SettingsApp";
import { isTauri } from "./lib/tauri";
import "./styles.css";

const isSettings = isTauri()
  ? getCurrentWindow().label === "settings"
  : new URLSearchParams(window.location.search).has("settings");

const root = document.getElementById("root");
if (!root) throw new Error("Missing root element");

createRoot(root).render(
  <StrictMode>{isSettings ? <SettingsApp /> : <DisplayApp />}</StrictMode>,
);
