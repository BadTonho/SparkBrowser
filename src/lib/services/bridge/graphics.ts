import { invoke, isTauriRuntime } from "./runtime";
import type { GraphicsStatus } from "../../types/graphics";
import type { ShutdownState } from "../../types/shutdown";

/** Browser preview has no GPU choice to make; it reports acceleration as on and nothing pending. */
export const previewGraphics: GraphicsStatus = { enabled: true, active: true, restartRequired: false };
export const fetchGraphicsStatus = () => (isTauriRuntime() ? invoke<GraphicsStatus>("get_graphics_status") : Promise.resolve(previewGraphics));
export const saveHardwareAcceleration = (enabled: boolean) => invoke<GraphicsStatus>("set_hardware_acceleration", { enabled });
export const requestGraphicsRestart = () => invoke<ShutdownState>("restart_for_graphics");
