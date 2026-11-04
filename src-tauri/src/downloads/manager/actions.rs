use super::*;
impl DownloadManager {
    pub async fn clear_finished(&self, app: &AppHandle) -> Result<Vec<String>, String> {
        let _operation = self.operations.lock().await;
        let mut tasks = self.tasks.lock().map_err(|e| e.to_string())?;
        let removed = tasks
            .iter()
            .filter(|(_, v)| !v.worker && !v.record.task.unfinished())
            .map(|(id, _)| id.clone())
            .collect::<Vec<_>>();
        let state = app.state::<BrowserState>();
        let db = state.db.lock().map_err(|e| e.to_string())?;
        let tx = db
            .as_ref()
            .ok_or("downloads.database_unavailable")?
            .unchecked_transaction()
            .map_err(|e| e.to_string())?;
        crate::database::downloads_repo::clear_downloads(&tx)?;
        for id in &removed {
            tx.execute("DELETE FROM download_jobs WHERE id=?1", [id])
                .map_err(|e| e.to_string())?;
        }
        tx.commit().map_err(|e| e.to_string())?;
        for id in &removed {
            tasks.remove(id);
        }
        drop(tasks);
        let _ = crate::security::emit_shell(app, "downloads-updated", ());
        Ok(removed)
    }
    pub async fn control(&self, app: &AppHandle, id: &str, action: &str) -> Result<(), String> {
        let _operation = self.operations.lock().await;
        let record = self.record(id)?;
        if record.task.source == "native"
            && action != "verify"
            && !self
                .tasks
                .lock()
                .map_err(|e| e.to_string())?
                .get(id)
                .is_some_and(|v| v.worker)
        {
            return crate::downloads::native::control(app, id, action).await;
        }
        match action {
            "resume" => self.start(app, id, false),
            "verify" => self.start(app, id, true),
            "pause" | "cancel" => {
                let worker = {
                    let mut tasks = self.tasks.lock().map_err(|e| e.to_string())?;
                    let live = tasks.get_mut(id).ok_or("downloads.not_found")?;
                    if !live.record.task.unfinished() && !live.worker {
                        return Ok(());
                    }
                    if live.worker {
                        let before = live.record.clone();
                        live.record.task.status = if action == "pause" {
                            "Pausing"
                        } else {
                            "Cancelling"
                        }
                        .into();
                        live.record.task.revision += 1;
                        live.record.refresh_capabilities(false);
                        if let Err(e) = Self::persist(app, &live.record) {
                            live.record = before;
                            return Err(e);
                        }
                        let _ = crate::security::emit_shell(
                            app,
                            "download-task-updated",
                            &live.record.task,
                        );
                        live.control
                            .requested
                            .store(if action == "pause" { 1 } else { 2 }, Ordering::Release);
                        live.control.changed.notify_one();
                    }
                    live.worker
                };
                if worker {
                    self.refresh_protection(app, &record.tab_id);
                } else {
                    if action == "cancel" {
                        let path = files::owned_partial(&record)?;
                        if path.exists() {
                            std::fs::remove_file(path).map_err(|e| e.to_string())?;
                        }
                    }
                    self.update_saved(app, id, |t| {
                        t.status = if action == "pause" {
                            "Paused"
                        } else {
                            "Cancelled"
                        }
                        .into()
                    })?;
                }
                Ok(())
            }
            "restart" => {
                if !record.task.can_restart {
                    return Err("downloads.busy".into());
                }
                let path = files::owned_partial(&record)?;
                if path.exists() {
                    std::fs::remove_file(path).map_err(|e| e.to_string())?;
                }
                self.edit(app, id, true, |r| {
                    r.confirmed = 0;
                    r.prefix_sha256 = crate::downloads::resumable::empty_hash();
                    r.etag = None;
                    r.final_url = None;
                    r.body_completed = false;
                    r.published = false;
                    r.task.received = 0;
                    r.task.total = None;
                    r.task.actual_sha256 = None;
                    r.task.status = "Paused".into();
                })?;
                self.start(app, id, false)
            }
            _ => Err("downloads.invalid_action".into()),
        }
    }
    pub async fn set_checksum(
        &self,
        app: &AppHandle,
        id: &str,
        value: Option<String>,
    ) -> Result<(), String> {
        let _operation = self.operations.lock().await;
        let value = crate::downloads::model::checksum(value)?;
        let record = self.record(id)?;
        if record.task.pending() {
            return Err("downloads.busy".into());
        }
        self.edit(app, id, true, |r| {
            r.task.expected_sha256 = value;
            r.task.actual_sha256 = None;
        })?;
        Ok(())
    }
}
