export interface ExtensionAction {
  default_title?: string | null;
  default_popup?: string | null;
  default_icon?: string | null;
}

export interface ExtensionDetails {
  id: string;
  name: string;
  version: string;
  description: string;
  path: string;
  enabled: boolean;
  installed_at: number;
  icon_data_url?: string | null;
  popup_url?: string | null;
  options_url?: string | null;
  permissions: string[];
}

