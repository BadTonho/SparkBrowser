/// <reference types="vite/client" />
// The release notes shipped with the app, bundled as text from the root `changelog/` folder.
export const changelogFiles: Record<string, string> = import.meta.glob("/changelog/*/*.md", { query: "?raw", import: "default", eager: true });
