export type SandboxStatus = "running" | "paused" | "stopped" | "crashed" | "draining";

export const SandboxStatuses: readonly SandboxStatus[] = [
  "running",
  "paused",
  "stopped",
  "crashed",
  "draining",
] as const;
