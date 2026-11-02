export const PIP_SHADOW_STORAGE_KEY = 'veloce_pip_window_shadow';

/** @typedef {{getItem:(key:string)=>string|null,setItem:(key:string,value:string)=>unknown}} ShadowStorage */
/** Missing, malformed and legacy values all opt out of the native shadow.
 * @param {ShadowStorage} storage
 */
export function readPipShadow(storage) {
  try {return storage.getItem(PIP_SHADOW_STORAGE_KEY) === 'true';} catch {return false;}
}

/** Serialize startup, user changes and imported preferences; publish confirmed values.
 * @param {{storage:ShadowStorage,apply:(enabled:boolean)=>Promise<boolean>,runtime:boolean,changed:(state:{value:boolean,available:boolean,busy:boolean})=>void,failed:(error:unknown)=>void,unavailable:(error:unknown)=>boolean}} options
 */
export function createPipShadowPreference({storage, apply, runtime, changed, failed, unavailable}) {
  let value=readPipShadow(storage),confirmed=false,available=false,pending=0,initialized=false;
  /** @type {Promise<void>|undefined} */
  let initialization;
  let tail=Promise.resolve();
  const publish=()=>changed({value,available,busy:pending>0});
  publish();
  /** @param {()=>boolean} request @param {boolean} persist */
  function submit(request,persist) {
    pending++;publish();
    const result=tail.then(async()=>{
      const enabled=request();
      try {
        if(runtime){value=confirmed=await apply(enabled);available=true;}
        else {value=enabled;available=false;}
        initialized=true;
      } catch(error) {
        if(unavailable(error)){available=false;value=enabled;initialized=true;}
        else {value=confirmed;failed(error);throw error;}
      }
      if(persist)storage.setItem(PIP_SHADOW_STORAGE_KEY,String(value));
      return value;
    }).finally(()=>{pending--;publish();});
    tail=result.then(()=>{},()=>{});
    return result;
  }
  return {
    initialize(){
      if(initialized)return tail;
      initialization??=submit(()=>readPipShadow(storage),false).then(()=>{}).catch(error=>{initialization=undefined;throw error;});
      return initialization.then(()=>tail);
    },
    /** @param {boolean} enabled */
    set(enabled){
      if(typeof enabled!=='boolean')return Promise.reject(new TypeError('Expected a boolean shadow preference'));
      return submit(()=>enabled,true);
    },
  };
}
