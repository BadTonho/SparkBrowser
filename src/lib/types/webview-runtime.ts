export interface WebviewRuntimeStatus {
  supported: boolean;
  currentVersion: string | null;
  availableVersion: string | null;
  restartRequired: boolean;
  revision: number;
  error: string | null;
}
