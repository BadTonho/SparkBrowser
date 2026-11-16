use super::*;
impl DownloadManager {
    pub fn start(&self, app: &AppHandle, id: &str, verify: bool) -> Result<(), String> {
        let record = self.record(id)?;
        if !verify {
            super::super::transfer::validate_download_url(&record.task.url)?;
            let state = app.state::<BrowserState>();
            let shield = *state
                .policies()
                .shield_enabled()
                .lock()
                .map_err(|e| e.to_string())?
                && crate::privacy::adblock_engine::site_is_protected(&state, &record.task.url)
                && record.task.source_page_url.as_deref().is_none_or(|source| {
                    crate::privacy::adblock_engine::site_is_protected(&state, source)
                });
            if crate::privacy::adblock_engine::should_block_url(&record.task.url, shield) {
                return Err("downloads.blocked".into());
            }
        }
        let (control, generation) = {
            let mut tasks = self.tasks.lock().map_err(|e| e.to_string())?;
            let live = tasks.get_mut(id).ok_or("downloads.not_found")?;
            if live.worker {
                return Err("downloads.busy".into());
            }
            if verify {
                if !live.record.task.can_verify {
                    return Err("downloads.not_available".into());
                }
            } else if !live.record.task.can_resume {
                return Err("downloads.restart_required".into());
            }
            live.worker = true;
            live.generation += 1;
            live.control = Arc::new(Control::default());
            (live.control.clone(), live.generation)
        };
        if let Err(error) = self.update_saved(app, id, |t| {
            t.status = if verify { "Verifying" } else { "Queued" }.into();
            t.error = None;
            t.verification_received = 0;
        }) {
            self.tasks
                .lock()
                .map_err(|e| e.to_string())?
                .get_mut(id)
                .unwrap()
                .worker = false;
            return Err(error);
        }
        let app = app.clone();
        let id = id.to_string();
        let slots = self.slots.clone();
        tauri::async_runtime::spawn(async move {
            let permit = tokio::select! {biased;_ = control.changed.notified()=>None,p=slots.acquire_owned()=>p.ok()};
            let result = if let Some(_permit) = permit {
                crate::downloads::resumable::run(&app, &id, &control, verify).await
            } else {
                Err("downloads.stopped".into())
            };
            app.state::<DownloadManager>()
                .finish_worker(&app, &id, generation, &control, result);
        });
        Ok(())
    }
    fn finish_worker(
        &self,
        app: &AppHandle,
        id: &str,
        generation: u64,
        control: &Control,
        result: Result<(), String>,
    ) {
        if !self.tasks.lock().is_ok_and(|mut tasks| {
            tasks.get_mut(id).is_some_and(|v| {
                if v.generation != generation {
                    return false;
                }
                v.worker = false;
                true
            })
        }) {
            return;
        }
        if let Err(error) = result {
            let requested = control.requested.load(Ordering::Acquire);
            let cancelled = requested == 2;
            if cancelled {
                if let Ok(record) = self.record(id) {
                    if let Ok(path) = files::owned_partial(&record) {
                        crate::error_log::ignored(std::fs::remove_file(path));
                    }
                }
            }
            crate::error_log::ignored(self.update_saved(app, id, |t| {
                t.status = if cancelled {
                    "Cancelled"
                } else if requested == 1 {
                    "Paused"
                } else if error.contains("restart_required")
                    || error.contains("file_changed")
                    || error.contains("file_missing")
                {
                    "RestartRequired"
                } else if error.contains("checksum_mismatch") {
                    "VerificationFailed"
                } else {
                    "Interrupted"
                }
                .into();
                t.error = if requested == 0 { Some(error) } else { None };
            }));
        }
        self.refresh_protection(app, &self.record(id).map(|r| r.tab_id).unwrap_or_default());
    }
}
