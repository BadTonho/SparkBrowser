import { mountPanelHost } from "../panels/panel-host";
import { downloadsSession, closeDownloadsDrawer, acknowledgeDownloadNotices } from "../../stores/downloadsStore";
import { onDownloadsPopoverAction, setDownloadsPopover } from "../../services/bridge/downloads-popover";
export const mountDownloadsHost = (root: HTMLElement, native: boolean) => mountPanelHost(root, native, { id: "downloads", sessionStore: downloadsSession, onTrigger: acknowledgeDownloadNotices, close: closeDownloadsDrawer, set: setDownloadsPopover, listen: onDownloadsPopoverAction });
