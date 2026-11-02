/** @template {{revision:number}} T @param {(state:T)=>void} apply */
export function createUpdateGate(apply) {
  let revision = -1;
  /** @param {T} state */
  return state => {
    if (state.revision <= revision) return false;
    revision = state.revision; apply(state); return true;
  };
}
