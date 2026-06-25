import { invoke } from "@tauri-apps/api/core";

export type Rgb = [number, number, number];

export type Mode =
  | { kind: "off" }
  | { kind: "solid"; rgb: Rgb }
  | { kind: "image"; path: string; zoom?: number; pan_x?: number; pan_y?: number }
  | { kind: "gif"; path: string; zoom?: number; pan_x?: number; pan_y?: number }
  | { kind: "text"; text: string; rgb: Rgb; scroll: boolean }
  | { kind: "anim"; id: string; speed: number; params?: Record<string, any> | null }
  | { kind: "raw" }
  | { kind: "loop"; ids: string[]; interval_secs: number };

export interface HistoryItem {
  path: string;
  favorite: boolean;
}

export interface Settings {
  mode: Mode;
  brightness: number;
  rotation: number;
  fps: number;
  autostart: boolean;
  minimize_to_tray: boolean;
  start_minimized: boolean;
  recent_images: HistoryItem[];
  recent_gifs: HistoryItem[];
  image_parameters?: Record<string, { zoom: number; pan_x: number; pan_y: number }>;
}

export interface Status {
  device_present: boolean;
  running: boolean;
  mode: Mode;
  conflict: boolean;
  loaded_path?: string | null;
}

export interface AnimInfo {
  id: string;
  label: string;
}

export const getStatus = () => invoke<Status>("get_status");
export const getSettings = () => invoke<Settings>("get_settings");
export const setMode = (mode: Mode) => invoke<void>("set_mode", { mode });
export const setBrightness = (pct: number) => invoke<void>("set_brightness", { pct });
export const setRotation = (deg: number) => invoke<void>("set_rotation", { deg });
export const setFps = (v: number) => invoke<void>("set_fps", { v });
export const setAutostart = (on: boolean) => invoke<void>("set_autostart", { on });
export const setMinimizeToTray = (on: boolean) => invoke<void>("set_minimize_to_tray", { on });
export const setStartMinimized = (on: boolean) => invoke<void>("set_start_minimized", { on });
export const listAnimations = () => invoke<AnimInfo[]>("list_animations");
export const pickFile = (kind: "image" | "gif") => invoke<string | null>("pick_file", { kind });
export const stopCmService = () => invoke<void>("stop_cm_service");
export const setRawFrame = (px: Rgb[]) => invoke<void>("set_raw_frame", { px });
export const toggleFavorite = (kind: "image" | "gif", path: string) => invoke<Settings>("toggle_favorite", { kind, path });
export const deleteHistory = (kind: "image" | "gif", path: string) => invoke<Settings>("delete_history", { kind, path });
