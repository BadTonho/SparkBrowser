(() => {
  if (window.top !== window) return;
  const BRIDGE_TOKEN = __GDRIVE_BRIDGE_TOKEN__;
  const FILE_NAME = "browser.veloce-backup.json";
  const CHUNK_SIZE = 400000;

  let capturedApiKey = null;
  let capturedBearer = null;
  let capturedAuthUser = "0";

  function inspectUrlForKey(rawUrl) {
    try {
      const u = new URL(rawUrl, location.origin);
      const k = u.searchParams.get("key");
      if (k && /^AIza[0-9A-Za-z_-]{30,45}$/.test(k)) {
        capturedApiKey = k;
      }
      const au = u.searchParams.get("authuser");
      if (au && /^\d+$/.test(au)) {
        capturedAuthUser = au;
      }
    } catch {}
  }

  // Observe page XHR and fetch calls to capture Google Drive Web's own API key and headers
  const origOpen = XMLHttpRequest.prototype.open;
  const origSetHeader = XMLHttpRequest.prototype.setRequestHeader;
  XMLHttpRequest.prototype.open = function (method, url, ...rest) {
    if (typeof url === "string") inspectUrlForKey(url);
    return origOpen.call(this, method, url, ...rest);
  };
  XMLHttpRequest.prototype.setRequestHeader = function (name, value) {
    if (typeof name === "string" && typeof value === "string") {
      const lower = name.toLowerCase();
      if (lower === "authorization" && value.startsWith("Bearer ")) {
        capturedBearer = value;
      } else if (lower === "x-goog-authuser" && /^\d+$/.test(value)) {
        capturedAuthUser = value;
      }
    }
    return origSetHeader.call(this, name, value);
  };

  const origFetch = window.fetch;
  if (typeof origFetch === "function") {
    window.fetch = function (input, init) {
      try {
        const url = typeof input === "string" ? input : input?.url;
        if (url) inspectUrlForKey(url);
        const headers = init?.headers;
        if (headers) {
          const auth =
            typeof headers.get === "function"
              ? headers.get("Authorization") || headers.get("authorization")
              : headers.Authorization || headers.authorization;
          if (typeof auth === "string" && auth.startsWith("Bearer ")) {
            capturedBearer = auth;
          }
        }
      } catch {}
      return origFetch.apply(this, arguments);
    };
  }

  function isLoginScreen() {
    const host = location.hostname.toLowerCase();
    const path = location.pathname;
    return (
      host === "accounts.google.com" ||
      path.startsWith("/ServiceLogin") ||
      path.startsWith("/v3/signin") ||
      path.startsWith("/AccountChooser")
    );
  }

  function extractApiKeyFromPage() {
    if (capturedApiKey) return capturedApiKey;
    try {
      const html = document.documentElement?.innerHTML || "";
      const match = html.match(/AIza[0-9A-Za-z_-]{35}/);
      if (match) {
        capturedApiKey = match[0];
        return capturedApiKey;
      }
    } catch {}
    return null;
  }

  function readSapisidCookie() {
    const cookies = document.cookie.split(";");
    for (const part of cookies) {
      const [rawName, ...rest] = part.trim().split("=");
      if (
        rawName === "SAPISID" ||
        rawName === "__Secure-3PAPISID" ||
        rawName === "__Secure-1PAPISID"
      ) {
        return decodeURIComponent(rest.join("="));
      }
    }
    return null;
  }

  async function buildAuthHeaders() {
    const headers = {
      "X-Origin": "https://drive.google.com",
      "X-Goog-AuthUser": capturedAuthUser || "0",
    };
    if (capturedBearer) {
      headers["Authorization"] = capturedBearer;
      return headers;
    }
    const sapisid = readSapisidCookie();
    if (!sapisid) return null;
    const timestamp = Math.floor(Date.now() / 1000);
    const origin = "https://drive.google.com";
    const input = `${timestamp} ${sapisid} ${origin}`;
    const bytes = new TextEncoder().encode(input);
    const digest = await crypto.subtle.digest("SHA-1", bytes);
    const hex = Array.from(new Uint8Array(digest))
      .map((b) => b.toString(16).padStart(2, "0"))
      .join("");
    headers["Authorization"] = `SAPISIDHASH ${timestamp}_${hex}`;
    return headers;
  }

  function sleep(ms) {
    return new Promise((resolve) => setTimeout(resolve, ms));
  }

  async function waitForDriveSession() {
    for (let i = 0; i < 30; i++) {
      if (isLoginScreen()) {
        throw new Error("backup.sync_gdrive_login_required");
      }
      const headers = await buildAuthHeaders();
      const apiKey = extractApiKeyFromPage();
      if (headers && (apiKey || capturedBearer)) {
        return { headers, apiKey };
      }
      await sleep(250);
    }
    if (isLoginScreen() || !readSapisidCookie()) {
      throw new Error("backup.sync_gdrive_login_required");
    }
    const headers = await buildAuthHeaders();
    const apiKey = extractApiKeyFromPage();
    if (!headers) {
      throw new Error("backup.sync_gdrive_login_required");
    }
    return { headers, apiKey };
  }

  function withKey(baseUrl, apiKey) {
    if (!apiKey) return baseUrl;
    const sep = baseUrl.includes("?") ? "&" : "?";
    return `${baseUrl}${sep}key=${encodeURIComponent(apiKey)}`;
  }

  async function findBackupFileId(headers, apiKey) {
    const q = encodeURIComponent(`name = '${FILE_NAME}' and trashed = false`);
    const url = withKey(
      `https://clients6.google.com/drive/v3/files?q=${q}&pageSize=1&spaces=drive&fields=files(id,name,modifiedTime)&orderBy=modifiedTime%20desc`,
      apiKey
    );
    const resp = await origFetch(url, {
      method: "GET",
      credentials: "include",
      headers,
    });
    if (resp.status === 401 || resp.status === 403) {
      throw new Error("backup.sync_gdrive_login_required");
    }
    if (!resp.ok) {
      throw new Error(`backup.sync_network_failed: HTTP ${resp.status}`);
    }
    const data = await resp.json();
    const files = Array.isArray(data?.files) ? data.files : [];
    return files.length > 0 ? files[0].id : null;
  }

  function decodeBase64Utf8(b64) {
    const binary = atob(b64);
    const bytes = new Uint8Array(binary.length);
    for (let i = 0; i < binary.length; i++) {
      bytes[i] = binary.charCodeAt(i);
    }
    return new TextDecoder().decode(bytes);
  }

  function encodeBase64Utf8(text) {
    const bytes = new TextEncoder().encode(text);
    let binary = "";
    const step = 0x8000;
    for (let i = 0; i < bytes.length; i += step) {
      binary += String.fromCharCode(...bytes.subarray(i, i + step));
    }
    return btoa(binary);
  }

  async function uploadBackupJson(jsonText) {
    const { headers, apiKey } = await waitForDriveSession();
    const existingId = await findBackupFileId(headers, apiKey);
    const boundary = "veloce_gdrive_boundary_" + Date.now();
    const metadata = existingId
      ? { mimeType: "application/json" }
      : { name: FILE_NAME, mimeType: "application/json" };
    const multipartBody =
      `--${boundary}\r\n` +
      `Content-Type: application/json; charset=UTF-8\r\n\r\n` +
      `${JSON.stringify(metadata)}\r\n` +
      `--${boundary}\r\n` +
      `Content-Type: application/json; charset=UTF-8\r\n\r\n` +
      `${jsonText}\r\n` +
      `--${boundary}--`;

    const baseUrl = existingId
      ? `https://clients6.google.com/upload/drive/v3/files/${encodeURIComponent(existingId)}?uploadType=multipart`
      : `https://clients6.google.com/upload/drive/v3/files?uploadType=multipart`;
    const url = withKey(baseUrl, apiKey);
    const method = existingId ? "PATCH" : "POST";

    const resp = await origFetch(url, {
      method,
      credentials: "include",
      headers: {
        ...headers,
        "Content-Type": `multipart/related; boundary=${boundary}`,
      },
      body: multipartBody,
    });
    if (resp.status === 401 || resp.status === 403) {
      throw new Error("backup.sync_gdrive_login_required");
    }
    if (!resp.ok) {
      throw new Error(`backup.sync_network_failed: HTTP ${resp.status}`);
    }
  }

  async function downloadBackupJson() {
    const { headers, apiKey } = await waitForDriveSession();
    const existingId = await findBackupFileId(headers, apiKey);
    if (!existingId) {
      throw new Error("backup.sync_file_not_found");
    }
    const url = withKey(
      `https://clients6.google.com/drive/v3/files/${encodeURIComponent(existingId)}?alt=media`,
      apiKey
    );
    const resp = await origFetch(url, {
      method: "GET",
      credentials: "include",
      headers,
    });
    if (resp.status === 401 || resp.status === 403) {
      throw new Error("backup.sync_gdrive_login_required");
    }
    if (resp.status === 404) {
      throw new Error("backup.sync_file_not_found");
    }
    if (!resp.ok) {
      throw new Error(`backup.sync_network_failed: HTTP ${resp.status}`);
    }
    return await resp.text();
  }

  async function sendBridgeMessage(action, params) {
    const query = new URLSearchParams({ token: BRIDGE_TOKEN, ...params });
    window.location.href = `veloce-internal://google-drive-sync/${action}?${query}`;
  }

  async function sendPayloadToRust(requestId, b64Payload) {
    if (b64Payload.length <= CHUNK_SIZE) {
      await sendBridgeMessage("complete", {
        request: requestId,
        payload: b64Payload,
      });
      return;
    }
    const total = Math.ceil(b64Payload.length / CHUNK_SIZE);
    for (let i = 0; i < total; i++) {
      const part = b64Payload.slice(i * CHUNK_SIZE, (i + 1) * CHUNK_SIZE);
      await sendBridgeMessage("chunk", {
        request: requestId,
        value: `${i}:${total}`,
        payload: part,
      });
      await sleep(20);
    }
    await sendBridgeMessage("complete", {
      request: requestId,
      payload: "",
    });
  }

  window.__veloceGDrivePush = async (token, requestId, b64Json) => {
    if (token !== BRIDGE_TOKEN) return;
    try {
      const jsonText = decodeBase64Utf8(b64Json);
      await uploadBackupJson(jsonText);
      await sendBridgeMessage("complete", { request: requestId, payload: "ok" });
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      await sendBridgeMessage("error", { request: requestId, data: msg.slice(0, 500) });
    }
  };

  window.__veloceGDrivePull = async (token, requestId) => {
    if (token !== BRIDGE_TOKEN) return;
    try {
      const jsonText = await downloadBackupJson();
      const b64 = encodeBase64Utf8(jsonText);
      await sendPayloadToRust(requestId, b64);
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      await sendBridgeMessage("error", { request: requestId, data: msg.slice(0, 500) });
    }
  };

  function reportPageState() {
    const status = isLoginScreen() ? "login_required" : "ready";
    void sendBridgeMessage("status", { reason: status });
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", reportPageState, { once: true });
  } else {
    setTimeout(reportPageState, 50);
  }
})();
