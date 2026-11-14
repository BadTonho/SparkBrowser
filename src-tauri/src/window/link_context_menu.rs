//! Route the native WebView2 link menu to a background browser tab.
#[cfg(windows)]
pub fn install(
    app: &tauri::AppHandle,
    view: &tauri::Webview,
    tab: &str,
    generation: &str,
) -> Result<(), String> {
    use webview2_com::{
        CoTaskMemPWSTR, ContextMenuRequestedEventHandler, CustomItemSelectedEventHandler,
        Microsoft::Web::WebView2::Win32::*,
    };
    use windows_core::{Interface, PWSTR};
    let app = app.clone();
    let tab = tab.to_owned();
    let generation = generation.to_owned();
    view.with_webview(move |platform| unsafe {
        let result = (|| {
            let core = platform.controller().CoreWebView2()?;
            let core11: ICoreWebView2_11 = core.cast()?;
            let environment: ICoreWebView2Environment9 = platform.environment().cast()?;
            let mut registration = 0;
            core11.add_ContextMenuRequested(
                &ContextMenuRequestedEventHandler::create(Box::new(move |_, args| {
                    let Some(args) = args else {
                        return Ok(());
                    };
                    let target = args.ContextMenuTarget()?;
                    let mut raw = PWSTR::null();
                    target.LinkUri(&mut raw)?;
                    if raw.is_null() {
                        return Ok(());
                    }
                    let value = CoTaskMemPWSTR::from(raw).to_string();
                    let Ok(url) = tauri::Url::parse(&value) else {
                        return Ok(());
                    };
                    if !crate::commands::tabs::popup_target_allowed(true, &url) {
                        return Ok(());
                    }
                    let items = args.MenuItems()?;
                    let mut count = 0;
                    items.Count(&mut count)?;
                    let mut translated_label = String::from("Open link in new tab");
                    for index in (0..count).rev() {
                        let item = items.GetValueAtIndex(index)?;
                        let mut name = PWSTR::null();
                        item.Name(&mut name)?;
                        if !name.is_null()
                            && CoTaskMemPWSTR::from(name).to_string() == "openLinkInNewTab"
                        {
                            let mut label = PWSTR::null();
                            item.Label(&mut label)?;
                            if !label.is_null() {
                                translated_label = CoTaskMemPWSTR::from(label).to_string();
                            }
                            items.RemoveValueAtIndex(index)?;
                        }
                    }
                    let label = CoTaskMemPWSTR::from(translated_label.as_str());
                    let item = environment.CreateContextMenuItem(
                        *label.as_ref().as_pcwstr(),
                        None,
                        COREWEBVIEW2_CONTEXT_MENU_ITEM_KIND_COMMAND,
                    )?;
                    let app = app.clone();
                    let tab = tab.clone();
                    let generation = generation.clone();
                    let mut token = 0;
                    item.add_CustomItemSelected(
                        &CustomItemSelectedEventHandler::create(Box::new(move |_, _| {
                            crate::commands::tabs::open_user_link(
                                app.clone(),
                                tab.clone(),
                                generation.clone(),
                                url.clone(),
                            );
                            Ok(())
                        })),
                        &mut token,
                    )?;
                    items.InsertValueAtIndex(0, &item)?;
                    Ok(())
                })),
                &mut registration,
            )?;
            Ok::<(), webview2_com::Error>(())
        })();
        if let Err(error) = result {
            eprintln!("Link context menu unavailable: {error}");
        }
    })
    .map_err(|e| e.to_string())
}

#[cfg(target_os = "linux")]
pub fn install(
    app: &tauri::AppHandle,
    view: &tauri::Webview,
    tab: &str,
    generation: &str,
) -> Result<(), String> {
    use gtk::gio;
    use gtk::prelude::*;
    use webkit2gtk::prelude::*;
    let app = app.clone();
    let tab = tab.to_owned();
    let generation = generation.to_owned();
    view.with_webview(move |platform| {
        platform
            .inner()
            .connect_context_menu(move |_, menu, _, hit| {
                let Some(uri) = hit.link_uri() else {
                    return false;
                };
                let Ok(url) = tauri::Url::parse(uri.as_str()) else {
                    return false;
                };
                if !crate::commands::tabs::popup_target_allowed(true, &url) {
                    return false;
                }
                for item in menu.items() {
                    if item.stock_action() == webkit2gtk::ContextMenuAction::OpenLinkInNewWindow {
                        menu.remove(&item);
                    }
                }
                let action = gio::SimpleAction::new("spark-open-link", None);
                let app = app.clone();
                let tab = tab.clone();
                let generation = generation.clone();
                action.connect_activate(move |_, _| {
                    crate::commands::tabs::open_user_link(
                        app.clone(),
                        tab.clone(),
                        generation.clone(),
                        url.clone(),
                    );
                });
                menu.prepend(&webkit2gtk::ContextMenuItem::from_gaction(
                    &action,
                    "Open link in new tab",
                    None,
                ));
                false
            });
    })
    .map_err(|e| e.to_string())
}

#[cfg(not(any(windows, target_os = "linux")))]
pub fn install(_: &tauri::AppHandle, _: &tauri::Webview, _: &str, _: &str) -> Result<(), String> {
    Ok(())
}
