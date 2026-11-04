import { mountPanelHost } from "../panels/panel-host";
import { settingsQuickSession, closeSettingsQuickMenu } from "../../stores/settingsQuickPopoverStore";
import { onSettingsQuickPopoverAction, setSettingsQuickPopover } from "../../services/bridge/settings-quick-popover";

export const mountSettingsQuickHost = (root: HTMLElement, native: boolean) => mountPanelHost(root, native, {
  id: "settings-quick",
  sessionStore: settingsQuickSession,
  close: closeSettingsQuickMenu,
  set: setSettingsQuickPopover,
  listen: onSettingsQuickPopoverAction,
});
