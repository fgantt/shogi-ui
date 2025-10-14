// Engine-related TypeScript types for the Tauri backend

export enum EngineStatus {
  Starting = "starting",
  Ready = "ready",
  Thinking = "thinking",
  Error = "error",
  Stopped = "stopped",
}

export interface EngineOption {
  name: string;
  option_type: string;
  default?: string;
  min?: string;
  max?: string;
  var: string[];
}

export interface EngineMetadata {
  name: string;
  author?: string;
  options: EngineOption[];
}

export interface EngineConfig {
  id: string;
  name: string;
  path: string;
  metadata?: EngineMetadata;
  is_builtin: boolean;
  enabled: boolean;
  last_used?: string;
  created_at: string;
}

export interface CommandResponse<T = any> {
  success: boolean;
  message?: string;
  data?: T;
}

export interface EngineHealthResult {
  id: string;
  name: string;
  status: "healthy" | "unhealthy" | "disabled";
  error?: string;
}

