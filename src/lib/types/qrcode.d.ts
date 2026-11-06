declare module "qrcode" {
  const QRCode: {
    toDataURL(text: string, options: { errorCorrectionLevel: "M"; margin: number; width: number; type: "image/png" }): Promise<string>;
  };
  export default QRCode;
}
