import type { CreateProfileForm } from "../../types/profiles";
import { isDownloadThumbnail } from "../download-visual";

const PHOTO_SIDE = 96;
const PHOTO_LIMIT = 5 * 1024 * 1024;
const MAX_PACKET = 90_000;
const encoder = new TextEncoder();

function encode(bytes: Uint8Array): string {
  let binary = "";
  for (const byte of bytes) binary += String.fromCharCode(byte);
  return btoa(binary).replaceAll("+", "-").replaceAll("/", "_").replace(/=+$/, "");
}

function decode(value: string): Uint8Array<ArrayBuffer> {
  return Uint8Array.from(atob(value.replaceAll("-", "+").replaceAll("_", "/")), char => char.charCodeAt(0));
}

export const newProfileFormKey = () => encode(crypto.getRandomValues(new Uint8Array(32)));

function importKey(key: string, usage: KeyUsage) {
  return crypto.subtle.importKey("raw", decode(key), "AES-GCM", false, [usage]);
}

function parameters(iv: Uint8Array<ArrayBuffer>, request: number) {
  return { name: "AES-GCM", iv, additionalData: encoder.encode(`spark:profile-create:${request}`) };
}

export async function sealProfileForm(form: CreateProfileForm, key: string, request: number): Promise<string> {
  const iv = crypto.getRandomValues(new Uint8Array(12));
  const secret = encoder.encode(JSON.stringify(form));
  try {
    const encrypted = new Uint8Array(await crypto.subtle.encrypt(parameters(iv, request), await importKey(key, "encrypt"), secret));
    const packet = new Uint8Array(iv.length + encrypted.length);
    packet.set(iv);
    packet.set(encrypted, iv.length);
    return encode(packet);
  } finally {
    secret.fill(0);
  }
}

export async function openProfileForm(packet: string, key: string, request: number): Promise<CreateProfileForm> {
  if (packet.length > MAX_PACKET) throw new Error("Profile form exceeds the transport limit");
  const bytes = decode(packet);
  const secret = new Uint8Array(await crypto.subtle.decrypt(parameters(bytes.slice(0, 12), request), await importKey(key, "decrypt"), bytes.subarray(12)));
  try {
    const form = JSON.parse(new TextDecoder().decode(secret)) as Partial<CreateProfileForm>;
    if (typeof form.name !== "string" || typeof form.password !== "string"
      || (form.avatar !== null && !isDownloadThumbnail(form.avatar))) {
      throw new Error("Invalid profile form");
    }
    return { name: form.name, avatar: form.avatar, password: form.password };
  } finally {
    secret.fill(0);
  }
}

/** The file path and original photo stay in the settings WebView. */
export async function profileThumbnail(file: File): Promise<string> {
  if (file.size > PHOTO_LIMIT) throw new Error("Profile photo exceeds the size limit");
  const header = new Uint8Array(await file.slice(0, 12).arrayBuffer());
  const signature = String.fromCharCode(...header);
  const type = signature.startsWith("\x89PNG\r\n\x1a\n") ? "image/png"
    : signature.startsWith("\xff\xd8\xff") ? "image/jpeg"
    : signature.startsWith("RIFF") && signature.slice(8) === "WEBP" ? "image/webp" : null;
  if (!type) throw new Error("Unsupported profile photo format");
  const url = URL.createObjectURL(file.slice(0, file.size, type));
  try {
    const photo = new Image();
    photo.src = url;
    await photo.decode();
    const side = Math.min(photo.naturalWidth, photo.naturalHeight);
    if (!side || photo.naturalWidth > 8192 || photo.naturalHeight > 8192) {
      throw new Error("Unsupported profile photo dimensions");
    }
    const canvas = document.createElement("canvas");
    canvas.width = PHOTO_SIDE;
    canvas.height = PHOTO_SIDE;
    const context = canvas.getContext("2d");
    if (!context) throw new Error("Profile photo canvas unavailable");
    context.drawImage(photo, (photo.naturalWidth - side) / 2, (photo.naturalHeight - side) / 2, side, side, 0, 0, PHOTO_SIDE, PHOTO_SIDE);
    return canvas.toDataURL("image/png");
  } finally {
    URL.revokeObjectURL(url);
  }
}
