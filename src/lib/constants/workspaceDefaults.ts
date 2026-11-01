import type { WorkspaceItem } from "../types/browser";

export const WORKSPACE_PALETTE_MIGRATION_VERSION = 1;

export const DEFAULT_WORKSPACES: WorkspaceItem[] = [
  {
    id: "ws-personal",
    name: "Personal",
    icon: "🌟",
    accent_primary: "#f5f5f5",
    accent_secondary: "#a3a3a3",
  },
  {
    id: "ws-work",
    name: "Work",
    icon: "💼",
    accent_primary: "#e5e5e5",
    accent_secondary: "#8a8a8a",
  },
  {
    id: "ws-study",
    name: "Study",
    icon: "📚",
    accent_primary: "#d4d4d4",
    accent_secondary: "#999999",
  },
  {
    id: "ws-gaming",
    name: "Gaming",
    icon: "🎮",
    accent_primary: "#c7c7c7",
    accent_secondary: "#737373",
  },
];

export const NEW_WORKSPACE_DEFAULT_COLORS = {
  accent_primary: "#e5e5e5",
  accent_secondary: "#9a9a9a",
};

const LEGACY_DEFAULT_WORKSPACE_COLORS: Record<
  string,
  { accent_primary: string; accent_secondary: string }
> = {
  "ws-personal": { accent_primary: "#6e8efb", accent_secondary: "#a777e3" },
  "ws-work": { accent_primary: "#38bdf8", accent_secondary: "#6366f1" },
  "ws-study": { accent_primary: "#34d399", accent_secondary: "#059669" },
  "ws-gaming": { accent_primary: "#f43f5e", accent_secondary: "#d946ef" },
};

/** Migrates only untouched colors on the four built-in workspace profiles. */
export function migrateLegacyDefaultWorkspaceColors(
  workspaces: WorkspaceItem[]
): WorkspaceItem[] {
  return workspaces.map((workspace) => {
    const legacy = LEGACY_DEFAULT_WORKSPACE_COLORS[workspace.id];
    const updatedDefault = DEFAULT_WORKSPACES.find(
      (item) => item.id === workspace.id
    );
    if (!legacy || !updatedDefault) return workspace;

    return {
      ...workspace,
      accent_primary:
        workspace.accent_primary.toLowerCase() === legacy.accent_primary
          ? updatedDefault.accent_primary
          : workspace.accent_primary,
      accent_secondary:
        workspace.accent_secondary.toLowerCase() === legacy.accent_secondary
          ? updatedDefault.accent_secondary
          : workspace.accent_secondary,
    };
  });
}
