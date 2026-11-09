use super::*;

#[cfg(windows)]
pub(super) fn build_gdrive_http_client() -> Result<reqwest::blocking::Client, String> {
    reqwest::blocking::Client::builder()
        .redirect(reqwest::redirect::Policy::limited(5))
        .timeout(Duration::from_secs(20))
        .connect_timeout(Duration::from_secs(10))
        .build()
        .map_err(|e| format!("backup.sync_network_failed: {e}"))
}

#[cfg(windows)]
pub(super) fn discover_gdrive_session_blocking(
    client: &reqwest::blocking::Client,
    cookie_header: &str,
    authuser: &str,
    cached_key: Option<String>,
) -> Result<(Vec<String>, Option<String>), String> {
    let mut detected_email: Option<String> = None;

    // 1. Fast check on Gmail Atom feed for exact connected email (same as New Tab Gmail widget)
    let feed_url = format!("https://mail.google.com/mail/u/{authuser}/feed/atom");
    if let Ok(resp) = client
        .get(&feed_url)
        .header(reqwest::header::COOKIE, cookie_header)
        .header(reqwest::header::USER_AGENT, USER_AGENT)
        .send()
    {
        if resp.status().is_success() {
            if let Ok(xml) = resp.text() {
                detected_email = extract_email_from_text(&xml);
            }
        }
    }

    let mut keys = Vec::new();
    if let Some(k) = cached_key {
        keys.push(k);
    }

    // 2. Fetch Google Drive page HTML to extract Drive's first-party API key (and email fallback)
    let drive_url = format!("https://drive.google.com/drive/u/{authuser}/my-drive");
    let resp = client
        .get(&drive_url)
        .header(reqwest::header::COOKIE, cookie_header)
        .header(reqwest::header::USER_AGENT, USER_AGENT)
        .send()
        .map_err(|e| format!("backup.sync_network_failed: {e}"))?;

    let final_url = resp.url().as_str().to_string();
    if final_url.contains("accounts.google.com") || final_url.contains("/ServiceLogin") {
        return Err("backup.sync_gdrive_login_required".into());
    }
    if resp.status().as_u16() == 401 || resp.status().as_u16() == 403 {
        return Err("backup.sync_gdrive_login_required".into());
    }

    let html = resp
        .text()
        .map_err(|e| format!("backup.sync_network_failed: {e}"))?;
    if detected_email.is_none() {
        detected_email = extract_email_from_text(&html);
    }
    for k in extract_api_keys_from_html(&html) {
        if !keys.contains(&k) {
            keys.push(k);
        }
    }

    Ok((keys, detected_email))
}

#[cfg(windows)]
pub(super) struct FoundGDriveFile {
    pub(super) id: Option<String>,
    pub(super) download_url: Option<String>,
    pub(super) api_version: String,
    pub(super) api_key: String,
    pub(super) folder_id: Option<String>,
}

#[cfg(windows)]
fn ensure_backup_folder_blocking(
    client: &reqwest::blocking::Client,
    cookie_header: &str,
    auth_header: &str,
    authuser: &str,
    api_keys: &[String],
    create_if_missing: bool,
) -> Result<(Option<String>, String, String), String> {
    if api_keys.is_empty() {
        return Err("backup.sync_gdrive_login_required".into());
    }

    let mut last_status = 0u16;
    let mut last_detail = String::new();
    let mut working_ver_and_key: Option<(String, String)> = None;

    for key in api_keys {
        for v2_ver in ["v2beta", "v2internal", "v2"] {
            let mut list_url =
                Url::parse(&format!("https://clients6.google.com/drive/{v2_ver}/files"))
                    .map_err(|e| e.to_string())?;
            list_url.query_pairs_mut()
                .append_pair(
                    "q",
                    &format!(
                        "title = '{SYNC_FOLDER_NAME}' and mimeType = '{FOLDER_MIME_TYPE}' and 'root' in parents and trashed = false"
                    ),
                )
                .append_pair("maxResults", "1")
                .append_pair("key", key);

            if let Ok(r) = client
                .get(list_url.as_str())
                .header(reqwest::header::COOKIE, cookie_header)
                .header(reqwest::header::AUTHORIZATION, auth_header)
                .header(reqwest::header::ORIGIN, "https://drive.google.com")
                .header(reqwest::header::REFERER, "https://drive.google.com/")
                .header(reqwest::header::USER_AGENT, USER_AGENT)
                .header("X-Goog-AuthUser", authuser)
                .send()
            {
                last_status = r.status().as_u16();
                if r.status().is_success() {
                    if let Ok(txt) = r.text() {
                        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&txt) {
                            let id = json["items"]
                                .as_array()
                                .and_then(|arr| arr.first())
                                .and_then(|item| item["id"].as_str())
                                .map(str::to_string);
                            if id.is_some() {
                                return Ok((id, v2_ver.to_string(), key.clone()));
                            }
                            if working_ver_and_key.is_none() {
                                working_ver_and_key = Some((v2_ver.to_string(), key.clone()));
                            }
                            break;
                        }
                    }
                } else if last_status == 401 {
                    return Err("backup.sync_gdrive_login_required".into());
                } else if let Ok(txt) = r.text() {
                    if let Ok(v) = serde_json::from_str::<serde_json::Value>(&txt) {
                        if let Some(m) = v["error"]["message"].as_str() {
                            last_detail = m.to_string();
                        }
                    }
                }
            }
        }

        if working_ver_and_key.is_none() {
            let mut list_v3 = Url::parse("https://clients6.google.com/drive/v3/files")
                .map_err(|e| e.to_string())?;
            list_v3
                .query_pairs_mut()
                .append_pair(
                    "q",
                    &format!(
                        "name = '{SYNC_FOLDER_NAME}' and mimeType = '{FOLDER_MIME_TYPE}' and 'root' in parents and trashed = false"
                    ),
                )
                .append_pair("pageSize", "1")
                .append_pair("key", key);

            if let Ok(r) = client
                .get(list_v3.as_str())
                .header(reqwest::header::COOKIE, cookie_header)
                .header(reqwest::header::AUTHORIZATION, auth_header)
                .header(reqwest::header::ORIGIN, "https://drive.google.com")
                .header(reqwest::header::REFERER, "https://drive.google.com/")
                .header(reqwest::header::USER_AGENT, USER_AGENT)
                .header("X-Goog-AuthUser", authuser)
                .send()
            {
                last_status = r.status().as_u16();
                if r.status().is_success() {
                    if let Ok(txt) = r.text() {
                        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&txt) {
                            let id = json["files"]
                                .as_array()
                                .and_then(|arr| arr.first())
                                .and_then(|item| item["id"].as_str())
                                .map(str::to_string);
                            if id.is_some() {
                                return Ok((id, "v3".to_string(), key.clone()));
                            }
                            working_ver_and_key = Some(("v3".to_string(), key.clone()));
                        }
                    }
                } else if last_status == 401 {
                    return Err("backup.sync_gdrive_login_required".into());
                } else if let Ok(txt) = r.text() {
                    if let Ok(v) = serde_json::from_str::<serde_json::Value>(&txt) {
                        if let Some(m) = v["error"]["message"].as_str() {
                            last_detail = m.to_string();
                        }
                    }
                }
            }
        }

        if working_ver_and_key.is_some() {
            break;
        }
    }

    let Some((ver, key)) = working_ver_and_key else {
        return Err(format!(
            "backup.sync_network_failed: Drive query HTTP {last_status} {last_detail}"
        ));
    };

    if !create_if_missing {
        return Ok((None, ver, key));
    }

    for candidate_ver in [ver.as_str(), "v2beta", "v2internal", "v2", "v3"] {
        let is_v2 = candidate_ver != "v3";
        let create_url =
            format!("https://clients6.google.com/drive/{candidate_ver}/files?key={key}");
        let body = build_folder_metadata_json(is_v2).to_string();

        if let Ok(r) = client
            .post(&create_url)
            .header(reqwest::header::COOKIE, cookie_header)
            .header(reqwest::header::AUTHORIZATION, auth_header)
            .header(reqwest::header::ORIGIN, "https://drive.google.com")
            .header(reqwest::header::REFERER, "https://drive.google.com/")
            .header(reqwest::header::USER_AGENT, USER_AGENT)
            .header("X-Goog-AuthUser", authuser)
            .header(
                reqwest::header::CONTENT_TYPE,
                "application/json; charset=UTF-8",
            )
            .body(body)
            .send()
        {
            if r.status().is_success() {
                if let Ok(txt) = r.text() {
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&txt) {
                        if let Some(folder_id) = json["id"].as_str() {
                            return Ok((
                                Some(folder_id.to_string()),
                                candidate_ver.to_string(),
                                key,
                            ));
                        }
                    }
                }
            }
        }
    }

    Ok((None, ver, key))
}

#[cfg(windows)]
fn query_backup_file_in_parent_blocking(
    client: &reqwest::blocking::Client,
    cookie_header: &str,
    auth_header: &str,
    authuser: &str,
    api_version: &str,
    api_key: &str,
    parent_id: Option<&str>,
) -> Result<Option<(String, Option<String>)>, String> {
    let is_v2 = api_version != "v3";
    let mut url = Url::parse(&format!(
        "https://clients6.google.com/drive/{api_version}/files"
    ))
    .map_err(|e| e.to_string())?;

    let query = match (is_v2, parent_id) {
        (true, Some(pid)) => {
            format!("title = '{SYNC_FILE_NAME}' and '{pid}' in parents and trashed = false")
        }
        (true, None) => format!("title = '{SYNC_FILE_NAME}' and trashed = false"),
        (false, Some(pid)) => {
            format!("name = '{SYNC_FILE_NAME}' and '{pid}' in parents and trashed = false")
        }
        (false, None) => format!("name = '{SYNC_FILE_NAME}' and trashed = false"),
    };

    {
        let mut qp = url.query_pairs_mut();
        qp.append_pair("q", &query);
        if is_v2 {
            qp.append_pair("maxResults", "1");
        } else {
            qp.append_pair("pageSize", "1");
        }
        qp.append_pair("key", api_key);
    }

    let resp = client
        .get(url.as_str())
        .header(reqwest::header::COOKIE, cookie_header)
        .header(reqwest::header::AUTHORIZATION, auth_header)
        .header(reqwest::header::ORIGIN, "https://drive.google.com")
        .header(reqwest::header::REFERER, "https://drive.google.com/")
        .header(reqwest::header::USER_AGENT, USER_AGENT)
        .header("X-Goog-AuthUser", authuser)
        .send()
        .map_err(|e| format!("backup.sync_network_failed: {e}"))?;

    if resp.status().as_u16() == 401 {
        return Err("backup.sync_gdrive_login_required".into());
    }
    if !resp.status().is_success() {
        return Ok(None);
    }

    let txt = resp
        .text()
        .map_err(|e| format!("backup.sync_network_failed: {e}"))?;
    let json = serde_json::from_str::<serde_json::Value>(&txt)
        .map_err(|e| format!("backup.sync_network_failed: {e}"))?;
    let arr_key = if is_v2 { "items" } else { "files" };
    let first = json[arr_key].as_array().and_then(|arr| arr.first());
    let Some(id) = first.and_then(|item| item["id"].as_str()) else {
        return Ok(None);
    };
    let dl = first
        .and_then(|item| item["downloadUrl"].as_str())
        .map(str::to_string);
    Ok(Some((id.to_string(), dl)))
}

#[cfg(windows)]
pub(super) fn find_backup_file_blocking(
    client: &reqwest::blocking::Client,
    cookie_header: &str,
    auth_header: &str,
    authuser: &str,
    api_keys: &[String],
    create_folder_if_missing: bool,
) -> Result<FoundGDriveFile, String> {
    let (folder_id, api_version, api_key) = ensure_backup_folder_blocking(
        client,
        cookie_header,
        auth_header,
        authuser,
        api_keys,
        create_folder_if_missing,
    )?;

    if let Some(ref fid) = folder_id {
        if let Some((file_id, dl_url)) = query_backup_file_in_parent_blocking(
            client,
            cookie_header,
            auth_header,
            authuser,
            &api_version,
            &api_key,
            Some(fid),
        )? {
            return Ok(FoundGDriveFile {
                id: Some(file_id),
                download_url: dl_url,
                api_version,
                api_key,
                folder_id,
            });
        }
    }

    // Legacy fallback during pull: if not yet inside the folder, check anywhere in My Drive
    if !create_folder_if_missing {
        if let Some((file_id, dl_url)) = query_backup_file_in_parent_blocking(
            client,
            cookie_header,
            auth_header,
            authuser,
            &api_version,
            &api_key,
            None,
        )? {
            return Ok(FoundGDriveFile {
                id: Some(file_id),
                download_url: dl_url,
                api_version,
                api_key,
                folder_id,
            });
        }
    }

    Ok(FoundGDriveFile {
        id: None,
        download_url: None,
        api_version,
        api_key,
        folder_id,
    })
}

#[cfg(windows)]
pub(super) fn upload_multipart_blocking(
    client: &reqwest::blocking::Client,
    cookie_header: &str,
    auth_header: &str,
    authuser: &str,
    found: &FoundGDriveFile,
    json_text: &str,
) -> Result<(), String> {
    let mut versions = vec![found.api_version.as_str()];
    for v in ["v2beta", "v2internal", "v2", "v3"] {
        if !versions.contains(&v) {
            versions.push(v);
        }
    }

    let boundary = format!(
        "veloce_gdrive_boundary_{}",
        crate::state::tab_model::TabInfo::current_epoch_secs()
    );
    let mut last_status = 0u16;
    let mut last_detail = String::new();
    let parent_id = found.folder_id.as_deref().unwrap_or("root");

    for ver in versions {
        let is_v2 = ver == "v2beta" || ver == "v2internal" || ver == "v2";
        let metadata = if is_v2 {
            if found.id.is_some() {
                serde_json::json!({ "title": SYNC_FILE_NAME, "mimeType": "application/json" })
            } else {
                serde_json::json!({
                    "title": SYNC_FILE_NAME,
                    "mimeType": "application/json",
                    "parents": [{ "id": parent_id }]
                })
            }
        } else if found.id.is_some() {
            serde_json::json!({ "mimeType": "application/json" })
        } else {
            serde_json::json!({
                "name": SYNC_FILE_NAME,
                "mimeType": "application/json",
                "parents": [parent_id]
            })
        };

        let body = format!(
            "--{boundary}\r\nContent-Type: application/json; charset=UTF-8\r\n\r\n{}\r\n--{boundary}\r\nContent-Type: application/json; charset=UTF-8\r\n\r\n{json_text}\r\n--{boundary}--",
            metadata
        );

        let url = match &found.id {
            Some(existing_id) => format!(
                "https://clients6.google.com/upload/drive/{ver}/files/{existing_id}?uploadType=multipart&key={}",
                found.api_key
            ),
            None => format!(
                "https://clients6.google.com/upload/drive/{ver}/files?uploadType=multipart&key={}",
                found.api_key
            ),
        };

        let builder = match (&found.id, is_v2) {
            (Some(_), true) => client.put(&url),
            (Some(_), false) => client.patch(&url),
            (None, _) => client.post(&url),
        };

        if let Ok(resp) = builder
            .header(reqwest::header::COOKIE, cookie_header)
            .header(reqwest::header::AUTHORIZATION, auth_header)
            .header(reqwest::header::ORIGIN, "https://drive.google.com")
            .header(reqwest::header::REFERER, "https://drive.google.com/")
            .header(reqwest::header::USER_AGENT, USER_AGENT)
            .header("X-Goog-AuthUser", authuser)
            .header(
                reqwest::header::CONTENT_TYPE,
                format!("multipart/related; boundary={boundary}"),
            )
            .body(body)
            .send()
        {
            last_status = resp.status().as_u16();
            if resp.status().is_success() {
                return Ok(());
            }
            if last_status == 401 {
                return Err("backup.sync_gdrive_login_required".into());
            }
            if let Ok(txt) = resp.text() {
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(&txt) {
                    if let Some(m) = v["error"]["message"].as_str() {
                        last_detail = m.to_string();
                    }
                }
            }
        }
    }

    Err(format!(
        "backup.sync_network_failed: Drive upload HTTP {last_status} {last_detail}"
    ))
}
