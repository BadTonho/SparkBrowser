/// Child WebViews cannot invoke application commands, including after navigation to a local URL.
pub fn shell_only(
    handler: impl Fn(tauri::ipc::Invoke<tauri::Wry>) -> bool + Send + Sync + 'static,
) -> impl Fn(tauri::ipc::Invoke<tauri::Wry>) -> bool + Send + Sync + 'static {
    move |invoke| {
        if invoke.message.webview_ref().label() != "main" {
            invoke
                .resolver
                .reject("Application commands are restricted to the browser shell");
            true
        } else {
            handler(invoke)
        }
    }
}
