export interface ShutdownReview {
  revision: number;
  tabs: { id: string; title: string; url: string; form: boolean; media: boolean; native_download: boolean }[];
  downloads: string[];
  native_downloads: string[];
}
export interface ShutdownState {
  purpose: "exit" | "install_update" | "restart_runtime";
  revision: number;
  phase: "idle" | "flushing" | "review" | "committing" | "error";
  attempt: string | null;
  update_version: string | null;
  review: ShutdownReview | null;
  error: string | null;
}
