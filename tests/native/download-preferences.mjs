/** Transfer scenarios own this explicit preference and retain it across restart.
 * The production default still opens the operating system's Save As dialog. */
export async function useAutomaticDownloads({ script, invoke }) {
  await script('localStorage.setItem("spark_ask_download_destination", "false")');
  await invoke('set_ask_download_destination', { enabled: false });
}
