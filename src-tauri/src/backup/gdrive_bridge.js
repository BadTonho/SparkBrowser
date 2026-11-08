(() => {
  if (location.protocol !== "https:") return;

  const BRIDGE_TOKEN = __GDRIVE_BRIDGE_TOKEN__;
  const FOLDER_NAME = "Veloce Browser Backup";
  const FOLDER_MIME_TYPE = "application/vnd.google-apps.folder";
  const FILE_NAME = "browser.veloce-backup.json";
  const CHUNK_SIZE = 400000;
  const nativeFetch = window.fetch.bind(window);

  // Child frame helper on clients6.google.com: captures Google Drive's own API keys
  // and acts as a same-origin fetch proxy when needed.
  if (window.top !== window) {
    if (location.hostname.toLowerCase() !== "clients6.google.com") return;

    function notifyTopKey(rawUrl, explicitKey) {
      try {
        if (explicitKey && /^AIza[0-9A-Za-z_-]{35}$/.test(explicitKey)) {
          window.top.postMessage(
            {
              __veloceGDriveBridge: BRIDGE_TOKEN,
              type: "captured_key",
              key: explicitKey,
              isDrive: true,
            },
            "https://drive.google.com"
          );
        }
        if (typeof rawUrl === "string") {
          const u = new URL(rawUrl, location.origin);
          const k = u.searchParams.get("key");
          if (k && /^AIza[0-9A-Za-z_-]{35}$/.test(k)) {
            window.top.postMessage(
              {
                __veloceGDriveBridge: BRIDGE_TOKEN,
                type: "captured_key",
                key: k,
                isDrive: u.pathname.includes("/drive/"),
              },
              "https://drive.google.com"
            );
          }
        }
      } catch {}
    }

    const origChildOpen = XMLHttpRequest.prototype.open;
    const origChildSetHeader = XMLHttpRequest.prototype.setRequestHeader;
    XMLHttpRequest.prototype.open = function (method, url, ...rest) {
      notifyTopKey(url, null);
      return origChildOpen.call(this, method, url, ...rest);
    };
    XMLHttpRequest.prototype.setRequestHeader = function (name, value) {
      if (typeof name === "string" && name.toLowerCase() === "x-goog-api-key" && typeof value === "string") {
        notifyTopKey(null, value.trim());
      }
      return origChildSetHeader.call(this, name, value);
    };

    window.addEventListener("message", async (event) => {
      if (event.origin !== "https://drive.google.com") return;
      const msg = event.data;
      if (!msg || msg.__veloceGDriveBridge !== BRIDGE_TOKEN || msg.type !== "proxy_fetch") return;
      try {
        const resp = await nativeFetch(msg.url, {
          method: msg.method || "GET",
          credentials: "include",
          headers: { ...(msg.headers || {}), "X-Origin": "https://drive.google.com" },
          body: msg.body ?? undefined,
        });
        const bodyText = await resp.text();
        window.top.postMessage(
          {
            __veloceGDriveBridge: BRIDGE_TOKEN,
            type: "proxy_fetch_result",
            id: msg.id,
            ok: resp.ok,
            status: resp.status,
            bodyText,
          },
          "https://drive.google.com"
        );
      } catch (err) {
        window.top.postMessage(
          {
            __veloceGDriveBridge: BRIDGE_TOKEN,
            type: "proxy_fetch_result",
            id: msg.id,
            ok: false,
            status: 0,
            error: err instanceof Error ? err.message : String(err),
          },
          "https://drive.google.com"
        );
      }
    });

    try {
      window.top.postMessage(
        { __veloceGDriveBridge: BRIDGE_TOKEN, type: "proxy_ready" },
        "https://drive.google.com"
      );
    } catch {}
    return;
  }

  const capturedApiKeys = [];
  let verifiedApiKey = null;
  let capturedBearer = null;
  let capturedAuthUser = null;
  let detectedEmail = null;
  let proxyFrameWindow = null;
  let nextProxyMsgId = 1;
  const pendingProxyFetches = new Map();

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

  window.addEventListener("message", (event) => {
    if (event.origin !== "https://clients6.google.com") return;
    const msg = event.data;
    if (!msg || msg.__veloceGDriveBridge !== BRIDGE_TOKEN) return;
    if (event.source) {
      proxyFrameWindow = event.source;
    }
    if (msg.type === "captured_key" && msg.key) {
      addApiKeyCandidate(msg.key, Boolean(msg.isDrive));
    } else if (msg.type === "proxy_fetch_result" && msg.id) {
      const resolver = pendingProxyFetches.get(msg.id);
      if (resolver) {
        pendingProxyFetches.delete(msg.id);
        resolver(msg);
      }
    }
  });

  function detectAuthUserFromPath() {
    const m = location.pathname.match(/\/u\/(\d+)(?:\/|$)/);
    if (m) return m[1];
    try {
      const au = new URLSearchParams(location.search).get("authuser");
      if (au && /^\d+$/.test(au)) return au;
    } catch {}
    return "0";
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
      path.startsWith("/AccountChooser") ||
      path.startsWith("/InteractiveLogin")
    );
  }

  function extractAccountEmail() {
    if (detectedEmail) return detectedEmail;
    try {
      const wizEmail = window.WIZ_global_data?.oPEP7c;
      if (typeof wizEmail === "string" && wizEmail.includes("@")) {
        detectedEmail = wizEmail.trim();
        return detectedEmail;
      }
    } catch {}
    try {
      const nodes = document.querySelectorAll("[aria-label*='@']");
      for (const node of nodes) {
        const label = node.getAttribute("aria-label") || "";
        const m = label.match(/([a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,})/);
        if (m) {
          detectedEmail = m[1];
          return detectedEmail;
        }
      }
    } catch {}
    try {
      const titleMatch = (document.title || "").match(/([a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,})/);
      if (titleMatch) {
        detectedEmail = titleMatch[1];
        return detectedEmail;
      }
    } catch {}
    try {
      const html = document.documentElement?.innerHTML || "";
      const wizMatch = html.match(/"oPEP7c"\s*:\s*"([a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,})"/);
      if (wizMatch) {
        detectedEmail = wizMatch[1];
        return detectedEmail;
      }
      const gmailMatch = html.match(/"([a-zA-Z0-9._%+-]+@(?:gmail|googlemail)\.com)"/);
      if (gmailMatch) {
        detectedEmail = gmailMatch[1];
        return detectedEmail;
      }
    } catch {}
    return null;
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
      if (window._drive_flag_initialData) {
        const driveText = JSON.stringify(window._drive_flag_initialData);
        const driveMatches = driveText.matchAll(/"(AIza[0-9A-Za-z_-]{35})"/g);
        for (const m of driveMatches) {
          addApiKeyCandidate(m[1], true);
        }
      }
    } catch {}
    try {
      for (const globalObj of [window.WIZ_global_data, window.IJ_values]) {
        if (globalObj) {
          const text = JSON.stringify(globalObj);
          const matches = text.matchAll(/"(AIza[0-9A-Za-z_-]{35})"/g);
          for (const m of matches) {
            addApiKeyCandidate(m[1], false);
          }
        }
      }
    } catch {}
    try {
      const html = document.documentElement?.innerHTML || "";
      const driveSpecific = html.matchAll(
        /clients6\.google\.com\/drive\/[^"']*?["'][^A]{0,120}?(AIza[0-9A-Za-z_-]{35})/g
      );
      for (const m of driveSpecific) {
        addApiKeyCandidate(m[1], true);
      }
      const allMatches = html.matchAll(/(?:["'=,\s])(AIza[0-9A-Za-z_-]{35})(?:["'&,\s\]}])/g);
      for (const m of allMatches) {
        addApiKeyCandidate(m[1], false);
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
    // Do NOT include X-Origin on direct CORS requests from https://drive.google.com;
    // the browser sends Origin: https://drive.google.com automatically, and sending
    // X-Origin causes Google ESF to reject with HTTP 400 ("Origin doesn't match Host for XD3").
    const headers = {
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

  async function smartFetch(url, options) {
    try {
      const resp = await nativeFetch(url, {
        method: options.method || "GET",
        credentials: "include",
        headers: options.headers || {},
        body: options.body ?? undefined,
      });
      return {
        ok: resp.ok,
        status: resp.status,
        text: () => resp.text(),
        json: async () => JSON.parse(await resp.text()),
      };
    } catch (directErr) {
      if (proxyFrameWindow) {
        const id = `pf-${nextProxyMsgId++}`;
        const proxyResult = await new Promise((resolve) => {
          const timer = setTimeout(() => {
            pendingProxyFetches.delete(id);
            resolve(null);
          }, 10000);
          pendingProxyFetches.set(id, (res) => {
            clearTimeout(timer);
            resolve(res);
          });
          try {
            proxyFrameWindow.postMessage(
              {
                __veloceGDriveBridge: BRIDGE_TOKEN,
                type: "proxy_fetch",
                id,
                url,
                method: options.method || "GET",
                headers: options.headers || {},
                body: options.body ?? null,
              },
              "https://clients6.google.com"
            );
          } catch {
            clearTimeout(timer);
            pendingProxyFetches.delete(id);
            resolve(null);
          }
        });
        if (proxyResult && proxyResult.status > 0) {
          return {
            ok: Boolean(proxyResult.ok),
            status: proxyResult.status,
            text: async () => proxyResult.bodyText || "",
            json: async () => JSON.parse(proxyResult.bodyText || "{}"),
          };
        }
      }
      throw directErr;
    }
  }

  async function waitForDriveSession() {
    for (let i = 0; i < 45; i++) {
      if (isLoginScreen()) {
        throw new Error("backup.sync_gdrive_login_required");
      }
      extractAccountEmail();
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

  async function maybeFetchAccountEmail(headers, apiVersion, apiKey) {
    if (extractAccountEmail()) return detectedEmail;
    try {
      const isV2 = apiVersion === "v2beta" || apiVersion === "v2internal" || apiVersion === "v2";
      const url = withKey(
        isV2
          ? `https://clients6.google.com/drive/${apiVersion}/about`
          : `https://clients6.google.com/drive/v3/about?fields=user(emailAddress)`,
        apiKey
      );
      const resp = await smartFetch(url, { method: "GET", headers });
      if (resp.ok) {
        const data = await resp.json();
        const email = data?.user?.emailAddress || data?.name;
        if (typeof email === "string" && email.includes("@")) {
          detectedEmail = email.trim();
        }
      }
    } catch {}
    return detectedEmail;
  }

  async function ensureBackupFolderId(headers, apiKeys, createIfMissing) {
    const candidates = apiKeys && apiKeys.length > 0 ? apiKeys : [null];
    let lastStatus = 0;
    let lastErrDetail = "";
    let workingVerAndKey = null;

    for (const candidateKey of candidates) {
      for (const v2Ver of ["v2beta", "v2internal", "v2"]) {
        try {
          const qV2 = encodeURIComponent(
            `title = '${FOLDER_NAME}' and mimeType = '${FOLDER_MIME_TYPE}' and 'root' in parents and trashed = false`
          );
          const urlV2 = withKey(
            `https://clients6.google.com/drive/${v2Ver}/files?q=${qV2}&maxResults=1`,
            candidateKey
          );
          const respV2 = await smartFetch(urlV2, { method: "GET", headers });
          if (respV2.ok) {
            if (candidateKey) verifiedApiKey = candidateKey;
            const data = await respV2.json();
            const items = Array.isArray(data?.items) ? data.items : [];
            await maybeFetchAccountEmail(headers, v2Ver, candidateKey);
            if (items.length > 0 && items[0].id) {
              return { folderId: items[0].id, apiVersion: v2Ver, apiKey: candidateKey };
            }
            if (!workingVerAndKey) {
              workingVerAndKey = { apiVersion: v2Ver, apiKey: candidateKey };
            }
            break;
          }
          lastStatus = respV2.status;
          if (respV2.status === 401) {
            throw new Error("backup.sync_gdrive_login_required");
          }
          try {
            const errTxt = await respV2.text();
            const errObj = JSON.parse(errTxt);
            if (errObj?.error?.message) lastErrDetail = errObj.error.message;
          } catch {}
        } catch (err) {
          if (err instanceof Error && err.message === "backup.sync_gdrive_login_required") {
            throw err;
          }
          lastErrDetail = err instanceof Error ? err.message : String(err);
        }
      }

      if (!workingVerAndKey) {
        try {
          const qV3 = encodeURIComponent(
            `name = '${FOLDER_NAME}' and mimeType = '${FOLDER_MIME_TYPE}' and 'root' in parents and trashed = false`
          );
          const urlV3 = withKey(
            `https://clients6.google.com/drive/v3/files?q=${qV3}&pageSize=1`,
            candidateKey
          );
          const respV3 = await smartFetch(urlV3, { method: "GET", headers });
          if (respV3.ok) {
            if (candidateKey) verifiedApiKey = candidateKey;
            const dataV3 = await respV3.json();
            const files = Array.isArray(dataV3?.files) ? dataV3.files : [];
            await maybeFetchAccountEmail(headers, "v3", candidateKey);
            if (files.length > 0 && files[0].id) {
              return { folderId: files[0].id, apiVersion: "v3", apiKey: candidateKey };
            }
            workingVerAndKey = { apiVersion: "v3", apiKey: candidateKey };
          } else {
            lastStatus = respV3.status;
            if (respV3.status === 401) {
              throw new Error("backup.sync_gdrive_login_required");
            }
            try {
              const errTxt = await respV3.text();
              const errObj = JSON.parse(errTxt);
              if (errObj?.error?.message) lastErrDetail = errObj.error.message;
            } catch {}
          }
        } catch (err) {
          if (err instanceof Error && err.message === "backup.sync_gdrive_login_required") {
            throw err;
          }
          lastErrDetail = err instanceof Error ? err.message : String(err);
        }
      }

      if (workingVerAndKey) break;
    }

    if (!workingVerAndKey) {
      if (lastStatus === 401) {
        throw new Error("backup.sync_gdrive_login_required");
      }
      throw new Error(
        `backup.sync_network_failed: Drive query failed (HTTP ${lastStatus}${lastErrDetail ? " " + lastErrDetail : ""})`
      );
    }

    const { apiVersion, apiKey } = workingVerAndKey;
    if (!createIfMissing) {
      return { folderId: null, apiVersion, apiKey };
    }

    const versionsToTry = [
      apiVersion,
      ...["v2beta", "v2internal", "v2", "v3"].filter((v) => v !== apiVersion),
    ];
    for (const ver of versionsToTry) {
      try {
        const isV2 = ver !== "v3";
        const createUrl = withKey(
          `https://clients6.google.com/drive/${ver}/files`,
          apiKey
        );
        const metadata = isV2
          ? { title: FOLDER_NAME, mimeType: FOLDER_MIME_TYPE, parents: [{ id: "root" }] }
          : { name: FOLDER_NAME, mimeType: FOLDER_MIME_TYPE, parents: ["root"] };
        const resp = await smartFetch(createUrl, {
          method: "POST",
          headers: { ...headers, "Content-Type": "application/json; charset=UTF-8" },
          body: JSON.stringify(metadata),
        });
        if (resp.ok) {
          const data = await resp.json();
          if (data?.id) {
            return { folderId: data.id, apiVersion: ver, apiKey };
          }
        }
      } catch {}
    }

    return { folderId: null, apiVersion, apiKey };
  }

  async function queryBackupFileInParent(headers, apiVersion, apiKey, parentId) {
    const isV2 = apiVersion !== "v3";
    const queryStr = isV2
      ? parentId
        ? `title = '${FILE_NAME}' and '${parentId}' in parents and trashed = false`
        : `title = '${FILE_NAME}' and trashed = false`
      : parentId
        ? `name = '${FILE_NAME}' and '${parentId}' in parents and trashed = false`
        : `name = '${FILE_NAME}' and trashed = false`;
    const limitParam = isV2 ? "maxResults=1" : "pageSize=1";
    const url = withKey(
      `https://clients6.google.com/drive/${apiVersion}/files?q=${encodeURIComponent(queryStr)}&${limitParam}`,
      apiKey
    );
    const resp = await smartFetch(url, { method: "GET", headers });
    if (resp.status === 401) {
      throw new Error("backup.sync_gdrive_login_required");
    }
    if (!resp.ok) return null;
    const data = await resp.json();
    const list = isV2 ? data?.items : data?.files;
    if (Array.isArray(list) && list.length > 0 && list[0].id) {
      return {
        id: list[0].id,
        downloadUrl: list[0].downloadUrl || null,
      };
    }
    return null;
  }

  async function findBackupFileId(headers, apiKeys, createFolderIfMissing) {
    const { folderId, apiVersion, apiKey } = await ensureBackupFolderId(
      headers,
      apiKeys,
      createFolderIfMissing
    );

    if (folderId) {
      const inFolder = await queryBackupFileInParent(headers, apiVersion, apiKey, folderId);
      if (inFolder) {
        return {
          id: inFolder.id,
          downloadUrl: inFolder.downloadUrl,
          apiVersion,
          apiKey,
          folderId,
        };
      }
    }

    if (!createFolderIfMissing) {
      const inAny = await queryBackupFileInParent(headers, apiVersion, apiKey, null);
      if (inAny) {
        return {
          id: inAny.id,
          downloadUrl: inAny.downloadUrl,
          apiVersion,
          apiKey,
          folderId,
        };
      }
    }

    return {
      id: null,
      downloadUrl: null,
      apiVersion,
      apiKey,
      folderId,
    };
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

  async function tryMultipartUpload(headers, apiKey, apiVersion, existingId, folderId, jsonText) {
    const boundary = "veloce_gdrive_boundary_" + Date.now();
    const isV2 = apiVersion === "v2beta" || apiVersion === "v2internal" || apiVersion === "v2";
    const parentId = folderId || "root";
    const metadata = isV2
      ? existingId
        ? { title: FILE_NAME, mimeType: "application/json" }
        : { title: FILE_NAME, mimeType: "application/json", parents: [{ id: parentId }] }
      : existingId
        ? { mimeType: "application/json" }
        : { name: FILE_NAME, mimeType: "application/json", parents: [parentId] };

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
    const method = existingId ? (isV2 ? "PUT" : "PATCH") : "POST";
    const reqHeaders = {
      ...headers,
      "Content-Type": `multipart/related; boundary=${boundary}`,
    };

    return await smartFetch(url, {
      method,
      headers: reqHeaders,
      body: multipartBody,
    });
  }

  async function uploadBackupJson(jsonText) {
    const { headers, apiKeys } = await waitForDriveSession();
    const { id: existingId, apiVersion, apiKey, folderId } = await findBackupFileId(
      headers,
      apiKeys,
      true
    );

    const versionsToTry = [
      apiVersion,
      ...["v2beta", "v2internal", "v2", "v3"].filter((v) => v !== apiVersion),
    ];
    let lastStatus = 0;
    let lastError = "";

    for (const ver of versionsToTry) {
      try {
        const resp = await tryMultipartUpload(
          headers,
          apiKey,
          ver,
          existingId,
          folderId,
          jsonText
        );
        if (resp.ok) {
          return;
        }
        lastStatus = resp.status;
        if (resp.status === 401) {
          throw new Error("backup.sync_gdrive_login_required");
        }
      } catch (err) {
        if (err instanceof Error && err.message === "backup.sync_gdrive_login_required") {
          throw err;
        }
        lastError = err instanceof Error ? err.message : String(err);
      }
    }

    throw new Error(
      `backup.sync_network_failed: Drive upload failed (HTTP ${lastStatus}${lastError ? " " + lastError : ""})`
    );
  }

  async function downloadBackupJson() {
    const { headers, apiKeys } = await waitForDriveSession();
    const { id: existingId, downloadUrl, apiVersion, apiKey } = await findBackupFileId(
      headers,
      apiKeys,
      false
    );
    if (!existingId) {
      throw new Error("backup.sync_file_not_found");
    }
    const url = withKey(
      `https://clients6.google.com/drive/${apiVersion}/files/${encodeURIComponent(existingId)}?alt=media`,
      apiKey
    );
    try {
      const resp = await smartFetch(url, {
        method: "GET",
        headers,
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
      try {
        const dlResp = await nativeFetch(downloadUrl, {
          method: "GET",
          credentials: "include",
        });
        if (dlResp.ok) {
          return await dlResp.text();
        }
      } catch {}
    }

    throw new Error("backup.sync_network_failed");
  }

  function normalizeBridgeError(err) {
    const raw = err instanceof Error ? err.message : String(err);
    if (raw.startsWith("backup.")) {
      return raw.slice(0, 500);
    }
    return `backup.sync_network_failed: ${raw}`.slice(0, 500);
  }

  async function sendBridgeMessage(action, params) {
    const cleanParams = { token: BRIDGE_TOKEN };
    for (const [k, v] of Object.entries(params)) {
      if (v !== undefined && v !== null && v !== "") {
        cleanParams[k] = String(v);
      }
    }
    const query = new URLSearchParams(cleanParams);
    window.location.href = `veloce-internal://google-drive-sync/${action}?${query}`;
  }

  async function sendPayloadToRust(requestId, b64Payload) {
    const email = extractAccountEmail();
    if (b64Payload.length <= CHUNK_SIZE) {
      await sendBridgeMessage("complete", {
        request: requestId,
        data: email || "",
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
      data: email || "",
      payload: "",
    });
  }

  window.__veloceGDrivePush = async (token, requestId, b64Json) => {
    if (token !== BRIDGE_TOKEN) return;
    try {
      const jsonText = decodeBase64Utf8(b64Json);
      await uploadBackupJson(jsonText);
      const email = extractAccountEmail();
      await sendBridgeMessage("complete", {
        request: requestId,
        data: email || "",
        payload: "ok",
      });
    } catch (err) {
      await sendBridgeMessage("error", {
        request: requestId,
        data: normalizeBridgeError(err),
      });
    }
  };

  window.__veloceGDrivePull = async (token, requestId) => {
    if (token !== BRIDGE_TOKEN) return;
    try {
      const jsonText = await downloadBackupJson();
      const b64 = encodeBase64Utf8(jsonText);
      await sendPayloadToRust(requestId, b64);
    } catch (err) {
      await sendBridgeMessage("error", {
        request: requestId,
        data: normalizeBridgeError(err),
      });
    }
  };

  let reportingInProgress = false;
  async function reportPageState() {
    if (reportingInProgress) return;
    reportingInProgress = true;
    try {
      if (isLoginScreen()) {
        await sendBridgeMessage("status", { reason: "login_required" });
        return;
      }
      if (location.hostname.toLowerCase() !== "drive.google.com") {
        return;
      }
      for (let i = 0; i < 15; i++) {
        if (isLoginScreen()) {
          await sendBridgeMessage("status", { reason: "login_required" });
          return;
        }
        const headers = await buildAuthHeaders();
        const keys = collectApiKeyCandidates();
        const email = extractAccountEmail();
        if (headers && keys.length > 0) {
          await sendBridgeMessage("status", {
            reason: "ready",
            data: email || "",
          });
          return;
        }
        await sleep(200);
      }
      const headers = await buildAuthHeaders();
      if (!headers) {
        await sendBridgeMessage("status", { reason: "login_required" });
      } else {
        const email = extractAccountEmail();
        await sendBridgeMessage("status", {
          reason: "ready",
          data: email || "",
        });
      }
    } finally {
      reportingInProgress = false;
    }
  }

  window.__veloceGDrivePing = (token) => {
    if (token !== BRIDGE_TOKEN) return;
    void reportPageState();
  };

  if (document.readyState === "complete") {
    setTimeout(() => void reportPageState(), 300);
  } else {
    window.addEventListener("load", () => setTimeout(() => void reportPageState(), 300), {
      once: true,
    });
    document.addEventListener(
      "DOMContentLoaded",
      () => {
        if (isLoginScreen()) void reportPageState();
      },
      { once: true }
    );
  }
})();
