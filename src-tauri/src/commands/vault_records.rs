//! Commands for the cards and contact details kept in the vault (Settings → Autofill). Same rules as
//! the login commands: only the trusted shell reaches them, private windows refuse them, and a card
//! number is shown only after the person proves who they are.
use crate::database::executor;
use crate::vault::hello;
use crate::vault::ops::{self, Proof};
use crate::vault::records::{self, Record, RecordListing, RecordRevealed, RecordSummary};
use crate::vault::service::VaultService;
use std::time::Instant;
use tauri::{AppHandle, State};
use zeroize::Zeroizing;

fn timestamp() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_secs() as i64)
}

#[tauri::command]
pub async fn vault_records_list(
    app: AppHandle,
    service: State<'_, VaultService>,
    kind: String,
) -> Result<RecordListing, String> {
    super::vault::allowed(&app)?;
    let service = service.inner().clone();
    executor::run(app, move |conn| {
        records::list(conn, &service, &kind, Instant::now())
    })
    .await
}

/// Opens one record. A card asks for the master password given here, or for Windows Hello when
/// `current_password` is empty; a contact opens with the vault.
#[tauri::command]
pub async fn vault_record_reveal(
    app: AppHandle,
    service: State<'_, VaultService>,
    id: String,
    current_password: Option<String>,
) -> Result<RecordRevealed, String> {
    super::vault::allowed(&app)?;
    let service = service.inner().clone();
    service.key(Instant::now())?;
    let kind = {
        let id = id.clone();
        executor::run(app.clone(), move |conn| records::kind_of(conn, &id)).await?
    };
    if kind != records::KIND_CARD {
        return executor::run(app, move |conn| {
            records::reveal(conn, &service, &id, Instant::now())
        })
        .await;
    }
    match current_password {
        Some(password) => {
            let password = Zeroizing::new(password);
            executor::run(app, move |conn| {
                ops::confirm_reveal(conn, &service, Proof::Password(&password), Instant::now())?;
                records::reveal(conn, &service, &id, Instant::now())
            })
            .await
        }
        None => {
            let secret = super::vault::hello_step("showing a saved card", hello::secret).await?;
            executor::run(app, move |conn| {
                ops::confirm_reveal(conn, &service, Proof::Hello(&secret), Instant::now())?;
                records::reveal(conn, &service, &id, Instant::now())
            })
            .await
        }
    }
}

#[tauri::command]
pub async fn vault_record_save(
    app: AppHandle,
    service: State<'_, VaultService>,
    id: Option<String>,
    record: Record,
) -> Result<RecordSummary, String> {
    super::vault::allowed(&app)?;
    let service = service.inner().clone();
    executor::run(app, move |conn| {
        records::save(
            conn,
            &service,
            id.as_deref(),
            record,
            timestamp(),
            Instant::now(),
        )
    })
    .await
}

#[tauri::command]
pub async fn vault_record_delete(
    app: AppHandle,
    service: State<'_, VaultService>,
    id: String,
) -> Result<(), String> {
    super::vault::allowed(&app)?;
    let service = service.inner().clone();
    executor::run(app, move |conn| {
        records::delete(conn, &service, &id, Instant::now())
    })
    .await
}
