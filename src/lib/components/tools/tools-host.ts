import { mountPanelHost } from "../panels/panel-host";
import { toolsSession, closeTools } from "../../stores/toolsStore";
import { onToolsPopoverAction, setToolsPopover } from "../../services/bridge/tools-popover";
export const mountToolsHost = (root: HTMLElement, native: boolean) => mountPanelHost(root, native, { id: "tools", sessionStore: toolsSession, close: closeTools, set: setToolsPopover, listen: onToolsPopoverAction });
