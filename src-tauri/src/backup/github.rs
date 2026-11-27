//! Sync to a private GitHub repository with a fine-grained personal access token.
//!
//! Spark writes only to its own branch (`spark-sync` by default, never the repository's default
//! branch). Each send replaces that branch with a single commit without parents that holds the sync
//! file and, under `versions/`, the newest [`sync_versions::KEEP`] earlier copies. The encrypted file
//! changes completely on every send, so a growing history would make the repository bigger each
//! time; with one commit, GitHub drops the old copies on its own.
//!
//! Before every send Spark checks that the repository is private, that the token can write, and
//! that the branch only holds Spark's files. Backup encryption is required for this target.
use super::{files, model::BackupDocument, sync::SYNC_FILE_NAME, sync_versions};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use serde::Deserialize;
use std::time::Duration;

const API: &str = "https://api.github.com";
pub const DEFAULT_BRANCH: &str = "spark-sync";
const VERSIONS_PREFIX: &str = "versions/";

/// `owner` and `name` of a repository, checked so they can be put in an API address as they are.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Repository {
    pub owner: String,
    pub name: String,
}

impl Repository {
    pub fn full_name(&self) -> String {
        format!("{}/{}", self.owner, self.name)
    }
}

fn valid_part(part: &str, max: usize) -> bool {
    !part.is_empty()
        && part.len() <= max
        && part != "."
        && part != ".."
        && part
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
}

/// Accepts `owner/name` or the repository's address (`https://github.com/owner/name(.git)`).
pub fn parse_repository(raw: &str) -> Result<Repository, String> {
    let trimmed = raw.trim().trim_end_matches('/');
    let path = trimmed
        .strip_prefix("https://github.com/")
        .or_else(|| trimmed.strip_prefix("http://github.com/"))
        .or_else(|| trimmed.strip_prefix("github.com/"))
        .unwrap_or(trimmed);
    let path = path.strip_suffix(".git").unwrap_or(path);
    let mut parts = path.split('/');
    let (Some(owner), Some(name), None) = (parts.next(), parts.next(), parts.next()) else {
        return Err("backup.github_invalid_repository".into());
    };
    if !valid_part(owner, 39) || !valid_part(name, 100) {
        return Err("backup.github_invalid_repository".into());
    }
    Ok(Repository {
        owner: owner.to_string(),
        name: name.to_string(),
    })
}

pub fn validate_branch(raw: &str) -> Result<String, String> {
    let branch = raw.trim();
    let valid = !branch.is_empty()
        && branch.len() <= 100
        && !branch.starts_with(['/', '.', '-'])
        && !branch.ends_with(['/', '.'])
        && !branch.contains("..")
        && !branch.contains("//")
        && !branch.ends_with(".lock")
        && branch
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | '/'));
    if valid {
        Ok(branch.to_string())
    } else {
        Err("backup.github_invalid_branch".into())
    }
}

pub fn validate_token(raw: &str) -> Result<String, String> {
    let token = raw.trim();
    if token.is_empty()
        || token.len() > 255
        || !token.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
    {
        return Err("backup.github_token_invalid".into());
    }
    Ok(token.to_string())
}

/// One file of the branch.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    pub path: String,
    pub sha: String,
    pub size: Option<u64>,
}

/// The files the next commit holds: the new file, the replaced one kept as a version when it
/// changed, and the newest earlier versions up to the limit. Returns `None` when the branch holds a
/// file Spark did not write, so it is never replaced.
pub fn plan_tree(
    existing: &[Entry],
    new_sha: &str,
    keep_version: bool,
    now: i64,
) -> Option<Vec<(String, String)>> {
    let mut versions: Vec<(i64, String, String)> = Vec::new();
    let mut current: Option<&Entry> = None;
    for entry in existing {
        if entry.path == SYNC_FILE_NAME {
            current = Some(entry);
            continue;
        }
        // Anything but a version Spark wrote means the branch is not Spark's alone.
        let time = entry
            .path
            .strip_prefix(VERSIONS_PREFIX)
            .and_then(sync_versions::parse_version)?;
        versions.push((time, entry.path.clone(), entry.sha.clone()));
    }
    if let Some(current) = current.filter(|current| keep_version && current.sha != new_sha) {
        let name = format!("{VERSIONS_PREFIX}{}", sync_versions::version_name(now));
        versions.retain(|(_, path, _)| *path != name);
        versions.push((now, name, current.sha.clone()));
    }
    versions.sort_by_key(|version| std::cmp::Reverse(version.0));
    versions.truncate(sync_versions::KEEP);
    let mut tree = vec![(SYNC_FILE_NAME.to_string(), new_sha.to_string())];
    tree.extend(versions.into_iter().map(|(_, path, sha)| (path, sha)));
    Some(tree)
}

/// The earlier versions kept on the branch, newest first.
pub fn versions_of(entries: &[Entry]) -> Vec<sync_versions::SyncVersion> {
    let mut versions: Vec<_> = entries
        .iter()
        .filter_map(|entry| {
            let name = entry.path.strip_prefix(VERSIONS_PREFIX)?;
            Some(sync_versions::SyncVersion {
                id: name.to_string(),
                created_at: sync_versions::parse_version(name)?,
                size: entry.size,
            })
        })
        .collect();
    versions.sort_by_key(|version| std::cmp::Reverse(version.created_at));
    versions
}

/// Calls to the GitHub API for one repository.
pub struct Client {
    http: reqwest::blocking::Client,
    token: String,
    repo: Repository,
}

#[derive(Deserialize)]
struct RepoInfo {
    private: bool,
    default_branch: String,
    #[serde(default)]
    permissions: Option<Permissions>,
}

#[derive(Deserialize)]
struct Permissions {
    push: bool,
}

#[derive(Deserialize)]
struct Sha {
    sha: String,
}

#[derive(Deserialize)]
struct RefInfo {
    object: Sha,
}

#[derive(Deserialize)]
struct CommitInfo {
    tree: Sha,
}

#[derive(Deserialize)]
struct TreeInfo {
    tree: Vec<TreeItem>,
    #[serde(default)]
    truncated: bool,
}

#[derive(Deserialize)]
struct TreeItem {
    path: String,
    #[serde(rename = "type")]
    kind: String,
    sha: String,
    #[serde(default)]
    size: Option<u64>,
}

#[derive(Deserialize)]
struct Blob {
    content: String,
    encoding: String,
}

fn read_body(response: reqwest::blocking::Response, limit: u64) -> Result<Vec<u8>, String> {
    use std::io::Read;
    let mut bytes = Vec::new();
    response
        .take(limit)
        .read_to_end(&mut bytes)
        .map_err(|e| format!("backup.sync_network_failed: {e}"))?;
    Ok(bytes)
}

/// The error code for a failed answer; `what` says what was being done, for the log.
fn status_error(response: &reqwest::blocking::Response) -> String {
    let status = response.status().as_u16();
    let rate_limited = response
        .headers()
        .get("x-ratelimit-remaining")
        .is_some_and(|value| value == "0");
    match status {
        401 => "backup.github_token_invalid".into(),
        403 | 429 if rate_limited || status == 429 => "backup.github_rate_limited".into(),
        403 => "backup.github_no_write".into(),
        404 => "backup.github_repo_not_found".into(),
        409 => "backup.github_repo_empty".into(),
        _ => format!("backup.sync_network_failed: HTTP {status}"),
    }
}

impl Client {
    pub fn new(token: &str, repo: Repository) -> Result<Self, String> {
        let http = reqwest::blocking::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(10))
            .user_agent("Spark-Browser-Sync")
            .build()
            .map_err(|e| format!("backup.sync_network_failed: {e}"))?;
        Ok(Self {
            http,
            token: validate_token(token)?,
            repo,
        })
    }

    fn url(&self, path: &str) -> String {
        format!("{API}/repos/{}/{}{path}", self.repo.owner, self.repo.name)
    }

    fn send(
        &self,
        method: reqwest::Method,
        path: &str,
        body: Option<serde_json::Value>,
    ) -> Result<reqwest::blocking::Response, String> {
        let mut request = self
            .http
            .request(method, self.url(path))
            .bearer_auth(&self.token)
            .header("Accept", "application/vnd.github+json")
            .header("X-GitHub-Api-Version", "2022-11-28");
        if let Some(body) = body {
            request = request
                .header("Content-Type", "application/json")
                .body(body.to_string());
        }
        request
            .send()
            .map_err(|e| format!("backup.sync_network_failed: {e}"))
    }

    fn json<T: serde::de::DeserializeOwned>(
        &self,
        method: reqwest::Method,
        path: &str,
        body: Option<serde_json::Value>,
        limit: u64,
    ) -> Result<T, String> {
        let response = self.send(method, path, body)?;
        if !response.status().is_success() {
            return Err(status_error(&response));
        }
        serde_json::from_slice(&read_body(response, limit)?)
            .map_err(|e| format!("backup.sync_network_failed: {e}"))
    }

    /// The repository must be private and writable with this token, and `branch` must not be its
    /// default branch (Spark replaces the whole branch on every send).
    pub fn check(&self, branch: &str) -> Result<(), String> {
        let info: RepoInfo = self.json(reqwest::Method::GET, "", None, 1 << 20)?;
        if !info.private {
            return Err("backup.github_repo_public".into());
        }
        if info
            .permissions
            .is_some_and(|permissions| !permissions.push)
        {
            return Err("backup.github_no_write".into());
        }
        if info.default_branch == branch {
            return Err("backup.github_default_branch".into());
        }
        Ok(())
    }

    /// The commit and files of `branch`, or `None` when the branch does not exist yet.
    pub fn read_branch(&self, branch: &str) -> Result<Option<(String, Vec<Entry>)>, String> {
        let response = self.send(
            reqwest::Method::GET,
            &format!("/git/ref/heads/{branch}"),
            None,
        )?;
        if response.status().as_u16() == 404 {
            return Ok(None);
        }
        if !response.status().is_success() {
            return Err(status_error(&response));
        }
        let head: RefInfo = serde_json::from_slice(&read_body(response, 1 << 20)?)
            .map_err(|e| format!("backup.sync_network_failed: {e}"))?;
        let commit: CommitInfo = self.json(
            reqwest::Method::GET,
            &format!("/git/commits/{}", head.object.sha),
            None,
            1 << 20,
        )?;
        let tree: TreeInfo = self.json(
            reqwest::Method::GET,
            &format!("/git/trees/{}?recursive=1", commit.tree.sha),
            None,
            8 << 20,
        )?;
        if tree.truncated {
            return Err("backup.github_branch_in_use".into());
        }
        let entries = tree
            .tree
            .into_iter()
            .filter(|item| item.kind == "blob")
            .map(|item| Entry {
                path: item.path,
                sha: item.sha,
                size: item.size,
            })
            .collect();
        Ok(Some((head.object.sha, entries)))
    }

    fn read_blob(&self, sha: &str) -> Result<Vec<u8>, String> {
        let blob: Blob = self.json(
            reqwest::Method::GET,
            &format!("/git/blobs/{sha}"),
            None,
            files::MAX_FILE_BYTES * 2 + 4096,
        )?;
        if blob.encoding != "base64" {
            return Err("backup.sync_network_failed: unexpected blob encoding".into());
        }
        let compact: String = blob.content.split_whitespace().collect();
        let bytes = STANDARD
            .decode(compact)
            .map_err(|e| format!("backup.sync_network_failed: {e}"))?;
        if bytes.len() as u64 > files::MAX_FILE_BYTES {
            return Err("backup.file_too_large".into());
        }
        Ok(bytes)
    }

    /// Writes a failure to the error log with the repository and step, then returns it.
    fn logged<T>(&self, step: &str, result: Result<T, String>) -> Result<T, String> {
        result.inspect_err(|error| {
            // An empty branch is the normal first state, not a failure.
            if error != "backup.sync_file_not_found" {
                crate::error_log::warn(&format!(
                    "backup: GitHub {step} failed for {}: {error}",
                    self.repo.full_name()
                ));
            }
        })
    }

    /// Sends `bytes` as the sync file. `keep_version` keeps the file it replaces as a version.
    pub fn push(&self, branch: &str, bytes: &[u8], keep_version: bool) -> Result<(), String> {
        self.logged("send", self.push_checked(branch, bytes, keep_version))
    }

    fn push_checked(&self, branch: &str, bytes: &[u8], keep_version: bool) -> Result<(), String> {
        self.check(branch)?;
        let existing = self.read_branch(branch)?;
        let blob: Sha = self.json(
            reqwest::Method::POST,
            "/git/blobs",
            Some(serde_json::json!({ "content": STANDARD.encode(bytes), "encoding": "base64" })),
            1 << 20,
        )?;
        let now = crate::state::tab_model::TabInfo::current_epoch_secs() as i64;
        let entries = existing
            .as_ref()
            .map(|(_, entries)| entries.as_slice())
            .unwrap_or_default();
        let plan = plan_tree(entries, &blob.sha, keep_version, now)
            .ok_or("backup.github_branch_in_use")?;
        let tree_items: Vec<_> = plan
            .iter()
            .map(|(path, sha)| {
                serde_json::json!({ "path": path, "mode": "100644", "type": "blob", "sha": sha })
            })
            .collect();
        let tree: Sha = self.json(
            reqwest::Method::POST,
            "/git/trees",
            Some(serde_json::json!({ "tree": tree_items })),
            1 << 20,
        )?;
        let commit: Sha = self.json(
            reqwest::Method::POST,
            "/git/commits",
            Some(serde_json::json!({ "message": "Spark sync", "tree": tree.sha, "parents": [] })),
            1 << 20,
        )?;
        let response = if existing.is_some() {
            self.send(
                reqwest::Method::PATCH,
                &format!("/git/refs/heads/{branch}"),
                Some(serde_json::json!({ "sha": commit.sha, "force": true })),
            )?
        } else {
            self.send(
                reqwest::Method::POST,
                "/git/refs",
                Some(
                    serde_json::json!({ "ref": format!("refs/heads/{branch}"), "sha": commit.sha }),
                ),
            )?
        };
        match response.status().as_u16() {
            200 | 201 => Ok(()),
            // A protected branch refuses to be replaced.
            422 => Err("backup.github_branch_protected".into()),
            _ => Err(status_error(&response)),
        }
    }

    /// The sync file of `branch`.
    pub fn pull(&self, branch: &str, password: Option<&str>) -> Result<BackupDocument, String> {
        self.logged(
            "download",
            (|| {
                let (_, entries) = self
                    .read_branch(branch)?
                    .ok_or("backup.sync_file_not_found")?;
                let entry = entries
                    .iter()
                    .find(|entry| entry.path == SYNC_FILE_NAME)
                    .ok_or("backup.sync_file_not_found")?;
                files::decode_with(&self.read_blob(&entry.sha)?, password)
            })(),
        )
    }

    pub fn list_versions(&self, branch: &str) -> Result<Vec<sync_versions::SyncVersion>, String> {
        Ok(self
            .read_branch(branch)?
            .map(|(_, entries)| versions_of(&entries))
            .unwrap_or_default())
    }

    pub fn read_version(
        &self,
        branch: &str,
        id: &str,
        password: Option<&str>,
    ) -> Result<BackupDocument, String> {
        if sync_versions::parse_version(id).is_none() {
            return Err("backup.sync_version_not_found".into());
        }
        let path = format!("{VERSIONS_PREFIX}{id}");
        let (_, entries) = self
            .read_branch(branch)?
            .ok_or("backup.sync_version_not_found")?;
        let entry = entries
            .iter()
            .find(|entry| entry.path == path)
            .ok_or("backup.sync_version_not_found")?;
        files::decode_with(&self.read_blob(&entry.sha)?, password)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(path: &str, sha: &str) -> Entry {
        Entry {
            path: path.into(),
            sha: sha.into(),
            size: Some(10),
        }
    }

    #[test]
    fn repositories_are_read_from_a_name_or_an_address() {
        let expected = Repository {
            owner: "ana".into(),
            name: "spark-data".into(),
        };
        for typed in [
            "ana/spark-data",
            " https://github.com/ana/spark-data ",
            "https://github.com/ana/spark-data.git",
            "github.com/ana/spark-data/",
        ] {
            assert_eq!(parse_repository(typed).unwrap(), expected, "{typed}");
        }
        for bad in [
            "ana",
            "ana/spark/data",
            "ana/../x",
            "a na/x",
            "ana/x?y",
            "/x",
            "",
        ] {
            assert!(parse_repository(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn branches_and_tokens_are_checked() {
        assert_eq!(validate_branch(" spark-sync ").unwrap(), "spark-sync");
        assert!(validate_branch("spark/sync").is_ok());
        for bad in ["", "-x", "a..b", "x/", "a b", "x.lock", "ref?"] {
            assert!(validate_branch(bad).is_err(), "{bad}");
        }
        assert!(validate_token("github_pat_11ABC_def").is_ok());
        assert!(validate_token("ghp abc").is_err());
        assert!(validate_token("").is_err());
    }

    #[test]
    fn a_send_keeps_the_replaced_file_and_the_newest_versions_only() {
        let mut existing = vec![entry(SYNC_FILE_NAME, "old")];
        for time in 1..=5 {
            existing.push(entry(
                &format!("versions/{}", sync_versions::version_name(time)),
                &format!("v{time}"),
            ));
        }
        let plan = plan_tree(&existing, "new", true, 100).unwrap();
        assert_eq!(plan[0], (SYNC_FILE_NAME.to_string(), "new".to_string()));
        let versions: Vec<_> = plan[1..].iter().map(|(_, sha)| sha.as_str()).collect();
        assert_eq!(versions, ["old", "v5", "v4", "v3", "v2"]);
    }

    #[test]
    fn an_unchanged_file_or_a_first_send_adds_no_version() {
        let plan = plan_tree(&[entry(SYNC_FILE_NAME, "same")], "same", true, 100).unwrap();
        assert_eq!(plan.len(), 1);
        let plan = plan_tree(&[entry(SYNC_FILE_NAME, "old")], "new", false, 100).unwrap();
        assert_eq!(plan.len(), 1);
        assert_eq!(plan_tree(&[], "new", true, 100).unwrap().len(), 1);
    }

    #[test]
    fn a_branch_with_other_files_is_never_replaced() {
        let existing = [entry(SYNC_FILE_NAME, "a"), entry("README.md", "b")];
        assert!(plan_tree(&existing, "new", true, 100).is_none());
        assert!(plan_tree(&[entry("versions/notes.txt", "c")], "new", true, 100).is_none());
    }

    #[test]
    fn versions_are_listed_newest_first() {
        let entries = [
            entry(SYNC_FILE_NAME, "a"),
            entry(
                &format!("versions/{}", sync_versions::version_name(10)),
                "b",
            ),
            entry(
                &format!("versions/{}", sync_versions::version_name(30)),
                "c",
            ),
        ];
        let listed: Vec<_> = versions_of(&entries)
            .into_iter()
            .map(|v| v.created_at)
            .collect();
        assert_eq!(listed, [30, 10]);
    }
}
