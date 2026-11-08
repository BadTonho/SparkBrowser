(() => {
  if (window.top !== window) return;
  if (location.protocol !== "https:") return;

  const BRIDGE_TOKEN = __GDRIVE_BRIDGE_TOKEN__;
  const FILE_NAME = "browser.veloce-backup.json";
  const CHUNK_SIZE = 400000;

  const capturedApiKeys = [];
  let verifiedApiKey = null;
  let capturedBearer = null;
  let capturedAuthUser = null;

  function addApiKeyCandidate(key, prioritize = false) {
    if (typeof key !== "string" || !/^AIza[0-9A-Za-z_-]{35}$/.test(key)) {
      return;
    }
    const existingIdx = capturedApiKeys.indexOf(key);
    if (existingIdx !== -1) {
      if (prioritize && existingIdx > 0) {
        capturedApiKeys.splice(existingIdx, 1);
        capturedApiKeys.unshift(key);
      }
      return;
    }
    if (prioritize) {
      capturedApiKeys.unshift(key);
    } else {
      capturedApiKeys.push(key);
    }
  }

  function detectAuthUserFromPath() {
    const m = location.pathname.match(/\/u\/(\d+)(?:\/|$)/);
    return m ? m[1] : "0";
  }

  function inspectUrlForKey(rawUrl) {
    try {
      const u = new URL(rawUrl, location.origin);
      const k = u.searchParams.get("key");
      if (k) {
        const isDriveUrl = u.pathname.includes("/drive/");
        addApiKeyCandidate(k, isDriveUrl);
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
      } else if (lower === "x-goog-api-key") {
        addApiKeyCandidate(value.trim(), true);
      }
    }
    return origSetHeader.call(this, name, value);
  };

  const nativeFetch = window.fetch.bind(window);
  window.fetch = function (input, init) {
    try {
      const url = typeof input === "string" ? input : input?.url;
      if (url) inspectUrlForKey(url);
      const headers = init?.headers;
      if (headers) {
        const getHeader = (name) =>
          typeof headers.get === "function"
            ? headers.get(name) || headers.get(name.toLowerCase())
            : headers[name] || headers[name.toLowerCase()];
        const auth = getHeader("Authorization");
        if (typeof auth === "string" && auth.startsWith("Bearer ")) {
          capturedBearer = auth;
        }
        const apiKey = getHeader("X-Goog-Api-Key");
        if (typeof apiKey === "string") {
          addApiKeyCandidate(apiKey.trim(), true);
        }
      }
    } catch {}
    return nativeFetch(input, init);
  };

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

  function collectApiKeyCandidates() {
    if (verifiedApiKey) {
      addApiKeyCandidate(verifiedApiKey, true);
    }
    try {
      if (typeof performance !== "undefined" && typeof performance.getEntriesByType === "function") {
        for (const entry of performance.getEntriesByType("resource")) {
          if (entry?.name) inspectUrlForKey(entry.name);
        }
      }
    } catch {}
    try {
      for (const globalObj of [window._drive_flag_initialData, window.WIZ_global_data, window.IJ_values]) {
        if (globalObj) {
          const text = JSON.stringify(globalObj);
          const matches = text.matchAll(/AIza[0-9A-Za-z_-]{35}/g);
          for (const m of matches) {
            addApiKeyCandidate(m[0], false);
          }
        }
      }
    } catch {}
    try {
      const html = document.documentElement?.innerHTML || "";
      const matches = html.matchAll(/AIza[0-9A-Za-z_-]{35}/g);
      for (const m of matches) {
        addApiKeyCandidate(m[0], false);
      }
    } catch {}
    return [...capturedApiKeys];
  }

  function readCookieMap() {
    const map = {};
    const parts = (document.cookie || "").split(";");
    for (const part of parts) {
      const idx = part.indexOf("=");
      if (idx > 0) {
        const name = part.slice(0, idx).trim();
        const val = part.slice(idx + 1).trim();
        try {
          map[name] = decodeURIComponent(val);
        } catch {
          map[name] = val;
        }
      }
    }
    return map;
  }

  async function sha1Hex(input) {
    const bytes = new TextEncoder().encode(input);
    const digest = await crypto.subtle.digest("SHA-1", bytes);
    return Array.from(new Uint8Array(digest))
      .map((b) => b.toString(16).padStart(2, "0"))
      .join("");
  }

  async function buildAuthHeaders() {
    const authUser = capturedAuthUser || detectAuthUserFromPath();
    const headers = {
      "X-Origin": "https://drive.google.com",
      "X-Goog-AuthUser": authUser,
    };
    if (capturedBearer) {
      headers["Authorization"] = capturedBearer;
      return headers;
    }
    const cookies = readCookieMap();
    const sapisid = cookies["SAPISID"] || cookies["__Secure-3PAPISID"] || cookies["__Secure-1PAPISID"];
    if (!sapisid) return null;

    const timestamp = Math.floor(Date.now() / 1000);
    const origin = "https://drive.google.com";
    const tokens = [];
    if (cookies["SAPISID"] || cookies["__Secure-3PAPISID"]) {
      const s = cookies["SAPISID"] || cookies["__Secure-3PAPISID"];
      tokens.push(`SAPISIDHASH ${timestamp}_${await sha1Hex(`${timestamp} ${s} ${origin}`)}`);
    }
    if (cookies["__Secure-1PAPISID"]) {
      const s1 = cookies["__Secure-1PAPISID"];
      tokens.push(`SAPISID1PHASH ${timestamp}_${await sha1Hex(`${timestamp} ${s1} ${origin}`)}`);
    }
    if (cookies["__Secure-3PAPISID"]) {
      const s3 = cookies["__Secure-3PAPISID"];
      tokens.push(`SAPISID3PHASH ${timestamp}_${await sha1Hex(`${timestamp} ${s3} ${origin}`)}`);
    }
    headers["Authorization"] = tokens.join(" ");
    return headers;
  }

  function sleep(ms) {
    return new Promise((resolve) => setTimeout(resolve, ms));
  }

  async function waitForDriveSession() {
    for (let i = 0; i < 40; i++) {
      if (isLoginScreen()) {
        throw new Error("backup.sync_gdrive_login_required");
      }
      const headers = await buildAuthHeaders();
      const apiKeys = collectApiKeyCandidates();
      if (headers && (apiKeys.length > 0 || capturedBearer)) {
        return { headers, apiKeys };
      }
      await sleep(200);
    }
    if (isLoginScreen()) {
      throw new Error("backup.sync_gdrive_login_required");
    }
    const headers = await buildAuthHeaders();
    const apiKeys = collectApiKeyCandidates();
    if (!headers) {
      throw new Error("backup.sync_gdrive_login_required");
    }
    return { headers, apiKeys };
  }

  function withKey(baseUrl, apiKey) {
    if (!apiKey) return baseUrl;
    const sep = baseUrl.includes("?") ? "&" : "?";
    return `${baseUrl}${sep}key=${encodeURIComponent(apiKey)}`;
  }

  async function findBackupFileId(headers, apiKeys) {
    const candidates = apiKeys && apiKeys.length > 0 ? apiKeys : [null];
    let lastStatus = 0;

    for (const candidateKey of candidates) {
      const reqHeaders = candidateKey
        ? { ...headers, "X-Goog-Api-Key": candidateKey }
        : headers;

      const qV2 = encodeURIComponent(`title = '${FILE_NAME}' and trashed = false`);
      const urlV2 = withKey(
        `https://clients6.google.com/drive/v2beta/files?q=${qV2}&maxResults=1&fields=items(id,title,modifiedDate,downloadUrl)`,
        candidateKey
      );
      const respV2 = await nativeFetch(urlV2, {
        method: "GET",
        credentials: "include",
        headers: reqHeaders,
      });
      if (respV2.ok) {
        if (candidateKey) verifiedApiKey = candidateKey;
        const data = await respV2.json();
        const items = Array.isArray(data?.items) ? data.items : [];
        return {
          id: items.length > 0 ? items[0].id : null,
          downloadUrl: items.length > 0 ? items[0].downloadUrl || null : null,
          apiVersion: "v2beta",
          apiKey: candidateKey,
        };
      }
      lastStatus = respV2.status;
      if (respV2.status === 401) {
        throw new Error("backup.sync_gdrive_login_required");
      }

      const qV3 = encodeURIComponent(`name = '${FILE_NAME}' and trashed = false`);
      const urlV3 = withKey(
        `https://clients6.google.com/drive/v3/files?q=${qV3}&pageSize=1&spaces=drive&fields=files(id,name,modifiedTime)&orderBy=modifiedTime%20desc`,
        candidateKey
      );
      const respV3 = await nativeFetch(urlV3, {
        method: "GET",
        credentials: "include",
        headers: reqHeaders,
      });
      if (respV3.ok) {
        if (candidateKey) verifiedApiKey = candidateKey;
        const dataV3 = await respV3.json();
        const files = Array.isArray(dataV3?.files) ? dataV3.files : [];
        return {
          id: files.length > 0 ? files[0].id : null,
          downloadUrl: null,
          apiVersion: "v3",
          apiKey: candidateKey,
        };
      }
      lastStatus = respV3.status;
      if (respV3.status === 401) {
        throw new Error("backup.sync_gdrive_login_required");
      }
    }

    if (lastStatus === 401 || lastStatus === 403) {
      throw new Error("backup.sync_gdrive_login_required");
    }
    throw new Error(`backup.sync_network_failed: HTTP ${lastStatus}`);
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

  async function tryMultipartUpload(headers, apiKey, apiVersion, existingId, jsonText) {
    const boundary = "veloce_gdrive_boundary_" + Date.now();
    const metadata =
      apiVersion === "v2beta"
        ? { title: FILE_NAME, mimeType: "application/json" }
        : existingId
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
      ? `https://clients6.google.com/upload/drive/${apiVersion}/files/${encodeURIComponent(existingId)}?uploadType=multipart`
      : `https://clients6.google.com/upload/drive/${apiVersion}/files?uploadType=multipart`;
    const url = withKey(baseUrl, apiKey);
    const method = existingId ? (apiVersion === "v2beta" ? "PUT" : "PATCH") : "POST";
    const reqHeaders = {
      ...headers,
      ...(apiKey ? { "X-Goog-Api-Key": apiKey } : {}),
      "Content-Type": `multipart/related; boundary=${boundary}`,
    };

    return await nativeFetch(url, {
      method,
      credentials: "include",
      headers: reqHeaders,
      body: multipartBody,
    });
  }

  async function uploadBackupJson(jsonText) {
    const { headers, apiKeys } = await waitForDriveSession();
    const { id: existingId, apiVersion, apiKey } = await findBackupFileId(headers, apiKeys);

    let resp = await tryMultipartUpload(headers, apiKey, apiVersion, existingId, jsonText);
    if (!resp.ok) {
      const altVersion = apiVersion === "v2beta" ? "v3" : "v2beta";
      resp = await tryMultipartUpload(headers, apiKey, altVersion, existingId, jsonText);
    }
    if (resp.status === 401 || resp.status === 403) {
      throw new Error("backup.sync_gdrive_login_required");
    }
    if (!resp.ok) {
      throw new Error(`backup.sync_network_failed: HTTP ${resp.status}`);
    }
  }

  async function downloadBackupJson() {
    const { headers, apiKeys } = await waitForDriveSession();
    const { id: existingId, downloadUrl, apiVersion, apiKey } = await findBackupFileId(
      headers,
      apiKeys
    );
    if (!existingId) {
      throw new Error("backup.sync_file_not_found");
    }
    const reqHeaders = apiKey ? { ...headers, "X-Goog-Api-Key": apiKey } : headers;
    const url = withKey(
      `https://clients6.google.com/drive/${apiVersion}/files/${encodeURIComponent(existingId)}?alt=media`,
      apiKey
    );
    try {
      const resp = await nativeFetch(url, {
        method: "GET",
        credentials: "include",
        headers: reqHeaders,
      });
      if (resp.ok) {
        return await resp.text();
      }
      if (resp.status === 401) {
        throw new Error("backup.sync_gdrive_login_required");
      }
      if (resp.status === 404) {
        throw new Error("backup.sync_file_not_found");
      }
    } catch (err) {
      if (
        err instanceof Error &&
        (err.message === "backup.sync_gdrive_login_required" ||
          err.message === "backup.sync_file_not_found")
      ) {
        throw err;
      }
    }

    if (downloadUrl) {
      const dlResp = await nativeFetch(downloadUrl, {
        method: "GET",
        credentials: "include",
      });
      if (dlResp.ok) {
        return await dlResp.text();
      }
    }

    throw new Error("backup.sync_network_failed");
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
      await sleep(50);
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
    if (isLoginScreen()) {
      void sendBridgeMessage("status", { reason: "login_required" });
      return;
    }
    if (location.hostname.toLowerCase() === "drive.google.com") {
      void sendBridgeMessage("status", { reason: "ready" });
    }
  }

  window.__veloceGDrivePing = (token) => {
    if (token !== BRIDGE_TOKEN) return;
    reportPageState();
  };

  if (document.readyState === "complete") {
    setTimeout(reportPageState, 250);
  } else {
    window.addEventListener("load", () => setTimeout(reportPageState, 250), { once: true });
    document.addEventListener(
      "DOMContentLoaded",
      () => {
        if (isLoginScreen()) reportPageState();
      },
      { once: true }
    );
  }
})();
