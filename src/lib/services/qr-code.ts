// The QR code library is read the first time a code is made, so it stays out of the main window's
// start-up code: the panel is opened rarely.
export async function makeQrPng(url: string): Promise<string> {
  const { default: QRCode } = await import("qrcode");
  return await QRCode.toDataURL(url, { errorCorrectionLevel: "M", margin: 2, width: 288, type: "image/png" });
}

export async function copyPngImage(dataUrl: string): Promise<void> {
  if (!navigator.clipboard?.write || typeof ClipboardItem === "undefined") throw new Error("clipboard_image_unavailable");
  const comma = dataUrl.indexOf(",");
  if (comma < 0) throw new Error("invalid_png_data");
  const binary = atob(dataUrl.slice(comma + 1));
  const bytes = Uint8Array.from(binary, char => char.charCodeAt(0));
  const blob = new Blob([bytes], { type: "image/png" });
  await navigator.clipboard.write([new ClipboardItem({ "image/png": blob })]);
}

export async function savePngImage(dataUrl: string): Promise<boolean> {
  const comma = dataUrl.indexOf(",");
  if (comma < 0) throw new Error("invalid_png_data");
  return await import("@tauri-apps/api/core").then(({ invoke }) => invoke<boolean>("save_qr_code_png", {
    pngBase64: dataUrl.slice(comma + 1),
    suggestedName: "qr-code.png",
  }));
}
