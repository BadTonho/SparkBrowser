/** @returns {(packet: import('../types/browser').TabSnapshot) => import('../types/browser').TabItem[] | undefined} */
export function createSnapshotGate() {
  let revision = 0;
  return packet => {
    if (Array.isArray(packet)) return packet;
    if (packet.revision <= revision) return;
    revision = packet.revision;
    return packet.tabs;
  };
}
