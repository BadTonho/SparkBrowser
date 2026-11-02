import {writable} from "svelte/store";
import {safeStorage} from "../services/storage.js";
import {createPipShadowPreference} from "../services/pip-shadow-preference.js";
import {setNativePipWindowShadow} from "../services/bridge/media";
import {isTauriRuntime} from "../services/bridge/runtime";
import {describeError,errorDetails} from "../services/user-errors.js";

export const pipWindowShadow=writable(false);
export const pipShadowAvailable=writable(false);
export const pipShadowBusy=writable(false);
export const pipShadowError=writable("");
const preference=createPipShadowPreference({
  storage:safeStorage,apply:setNativePipWindowShadow,runtime:isTauriRuntime(),
  changed:(state:{value:boolean;available:boolean;busy:boolean})=>{pipWindowShadow.set(state.value);pipShadowAvailable.set(state.available);pipShadowBusy.set(state.busy);},
  failed:(error:unknown)=>pipShadowError.set(describeError(error)),
  unavailable:(error:unknown)=>errorDetails(error).code==="media.unavailable",
});
export const initializePipAppearance=()=>preference.initialize();
export async function setPipWindowShadow(enabled:boolean):Promise<void> {
  pipShadowError.set("");await preference.set(enabled);
}
