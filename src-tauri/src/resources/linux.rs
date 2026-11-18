//! Linux: the web engine's processes get a lower priority, which is a weaker control than a cap on Windows (it
//! only matters when the machine is busy), and the processor time they used is read from `/proc`.
//!
//! A process cannot raise its own priority back without privileges, so removing the limit lets new web
//! processes start at the normal priority but leaves the running ones lowered until the browser restarts.
//! The engine has no control over the pages' network speed, so only the browser's own downloads are limited.
use super::Limits;
use std::collections::{HashMap, HashSet};
use tauri::{AppHandle, Webview};

pub const SUPPORTS_CPU_LIMIT: bool = true;
pub const CPU_LIMIT_IS_PRIORITY: bool = true;
pub const SUPPORTS_NETWORK_LIMIT: bool = false;
pub const SUPPORTS_NETWORK_MONITOR: bool = false;

/// Priority given to the web processes while a processor limit is set (0 is normal, 19 the lowest).
const LOWERED: i32 = 10;

/// `(pid, parent, processor ticks)` of every process, read from `/proc`.
fn processes() -> Vec<(u32, u32, u64)> {
    let mut found = Vec::new();
    let Ok(entries) = std::fs::read_dir("/proc") else {
        return found;
    };
    for entry in entries.flatten() {
        let Some(pid) = entry
            .file_name()
            .to_str()
            .and_then(|name| name.parse::<u32>().ok())
        else {
            continue;
        };
        let Ok(stat) = std::fs::read_to_string(format!("/proc/{pid}/stat")) else {
            continue;
        };
        // "pid (comm) state ppid ..." - the name may contain spaces and parentheses.
        let Some((_, rest)) = stat.rsplit_once(") ") else {
            continue;
        };
        let fields: Vec<&str> = rest.split_whitespace().collect();
        let parent = fields.get(1).and_then(|v| v.parse().ok());
        let user: Option<u64> = fields.get(11).and_then(|v| v.parse().ok());
        let system: Option<u64> = fields.get(12).and_then(|v| v.parse().ok());
        if let (Some(parent), Some(user), Some(system)) = (parent, user, system) {
            found.push((pid, parent, user + system));
        }
    }
    found
}

/// The processes that descend from this one: the web engine's.
fn descendants(all: &[(u32, u32, u64)]) -> Vec<(u32, u64)> {
    let own = std::process::id();
    let parents: HashMap<u32, u32> = all.iter().map(|(pid, parent, _)| (*pid, *parent)).collect();
    let mut family = HashSet::from([own]);
    loop {
        let before = family.len();
        for (pid, parent) in &parents {
            if family.contains(parent) {
                family.insert(*pid);
            }
        }
        if family.len() == before {
            break;
        }
    }
    all.iter()
        .filter(|(pid, _, _)| *pid != own && family.contains(pid))
        .map(|(pid, _, ticks)| (*pid, *ticks))
        .collect()
}

pub fn cpu_seconds() -> Option<f64> {
    let per_second = unsafe { libc::sysconf(libc::_SC_CLK_TCK) };
    if per_second <= 0 {
        return None;
    }
    let all = processes();
    let ticks: u64 = descendants(&all).iter().map(|(_, ticks)| *ticks).sum();
    Some(ticks as f64 / per_second as f64)
}

pub fn apply(_app: &AppHandle, limits: Limits, _monitoring: bool) -> Result<(), String> {
    if limits.cpu_percent == 0 {
        return Ok(());
    }
    let all = processes();
    for (pid, _) in descendants(&all) {
        // A failure (the process ended, or it is not ours to change) is not an error for the user.
        unsafe {
            libc::setpriority(libc::PRIO_PROCESS, pid as libc::id_t, LOWERED);
        }
    }
    Ok(())
}

pub fn attach(_view: &Webview, _limits: Limits, _monitoring: bool) {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_family_follows_parents_and_never_includes_this_process() {
        let own = std::process::id();
        let all = vec![(own, 1, 0), (own + 1, own, 10), (own + 2, own + 1, 20), (own + 3, 1, 99)];
        let mut family: Vec<u32> = descendants(&all).into_iter().map(|(pid, _)| pid).collect();
        family.sort_unstable();
        assert_eq!(family, vec![own + 1, own + 2]);
    }
}
