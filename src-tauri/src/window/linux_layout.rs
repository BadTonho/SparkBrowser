#[cfg(target_os = "linux")]
use std::cell::RefCell;
#[cfg(target_os = "linux")]
use std::collections::HashMap;
#[cfg(target_os = "linux")]
use tauri::{AppHandle, Manager, WebviewWindow};

#[cfg(target_os = "linux")]
struct LinuxLayoutState {
    /// The GTK window that owns this layout; a child belongs to the layout of its toplevel.
    window: gtk::Widget,
    overlay: gtk::Overlay,
    vbox: gtk::Box,
}

/// The overlay and box of the window a widget sits in.
#[cfg(target_os = "linux")]
fn layout_of(widget: &gtk::Widget) -> Option<(gtk::Box, gtk::Overlay)> {
    use gtk::prelude::*;
    let toplevel = widget.toplevel()?;
    LAYOUT_CONTAINERS.with(|c| {
        c.borrow()
            .values()
            .find(|state| state.window == toplevel)
            .map(|state| (state.vbox.clone(), state.overlay.clone()))
    })
}

#[cfg(target_os = "linux")]
thread_local! {
    /// One layout per browser window, by window id.
    static LAYOUT_CONTAINERS: RefCell<HashMap<String, LinuxLayoutState>> = RefCell::new(HashMap::new());
    static LAYOUT_BOUNDS: RefCell<HashMap<String, gtk::gdk::Rectangle>> = RefCell::new(HashMap::new());
}

#[cfg(target_os = "linux")]
pub fn setup(window: &WebviewWindow) -> Result<(), String> {
    use gtk::prelude::*;
    use gtk::OverlaySignals;

    let gtk_window = window.gtk_window().map_err(|e| e.to_string())?;
    let vbox = window.default_vbox().map_err(|e| e.to_string())?;

    if let Some(parent) = vbox.parent() {
        if parent == gtk_window.clone().upcast::<gtk::Widget>() {
            gtk_window.remove(&vbox);
        }
    }

    let overlay = gtk::Overlay::new();
    gtk_window.add(&overlay);
    overlay.add(&vbox);

    overlay.connect_get_child_position(|_overlay, widget| {
        use gtk::prelude::WidgetExt;
        let name = widget.widget_name();
        LAYOUT_BOUNDS.with(|cell| cell.try_borrow().ok()?.get(name.as_str()).copied())
    });

    overlay.show_all();

    let window_widget = gtk_window.upcast::<gtk::Widget>();
    LAYOUT_CONTAINERS.with(|cell| {
        cell.borrow_mut().insert(
            window.label().to_string(),
            LinuxLayoutState {
                window: window_widget,
                overlay,
                vbox,
            },
        );
    });

    Ok(())
}

#[cfg(target_os = "linux")]
pub fn update_bounds(
    app: &AppHandle,
    label: &str,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    visible: bool,
) -> Result<(), String> {
    let label_str = label.to_string();
    let Some(webview) = app.get_webview(label) else {
        return Ok(());
    };

    webview
        .with_webview(move |platform| {
            use gtk::prelude::*;
            let widget = platform.inner().upcast::<gtk::Widget>();
            widget.set_widget_name(&label_str);

            // 1. Update bounds map first (short-lived borrow)
            LAYOUT_BOUNDS.with(|cell| {
                if let Ok(mut bounds) = cell.try_borrow_mut() {
                    bounds.insert(
                        label_str.clone(),
                        gtk::gdk::Rectangle::new(x, y, width, height),
                    );
                }
            });

            // 2. Retrieve containers (short-lived borrow)
            let containers = layout_of(&widget);

            // 3. Perform GTK widget tree modifications without holding any RefCell borrow
            if let Some((vbox, overlay)) = containers {
                if let Some(parent) = widget.parent() {
                    if parent == vbox.clone().upcast::<gtk::Widget>() {
                        vbox.remove(&widget);
                        overlay.add_overlay(&widget);
                        overlay.set_overlay_pass_through(&widget, false);
                    }
                }

                if visible {
                    widget.show();
                } else {
                    widget.hide();
                }

                overlay.queue_resize();
            }
        })
        .map_err(|e| e.to_string())
}

#[cfg(target_os = "linux")]
pub fn set_visible(app: &AppHandle, label: &str, visible: bool) -> Result<(), String> {
    let Some(webview) = app.get_webview(label) else {
        return Ok(());
    };

    webview
        .with_webview(move |platform| {
            use gtk::prelude::*;
            let widget = platform.inner().upcast::<gtk::Widget>();

            if visible {
                widget.show();
            } else {
                widget.hide();
            }

            if let Some((_, overlay)) = layout_of(&widget) {
                overlay.queue_resize();
            }
        })
        .map_err(|e| e.to_string())
}

/// Drops the layout of a closed window.
#[cfg(target_os = "linux")]
pub fn forget_window(window_id: &str) {
    LAYOUT_CONTAINERS.with(|c| c.borrow_mut().remove(window_id));
}

#[cfg(target_os = "linux")]
pub fn remove_child(_app: &AppHandle, label: &str) {
    let label = label.to_string();
    LAYOUT_BOUNDS.with(|cell| {
        if let Ok(mut bounds) = cell.try_borrow_mut() {
            bounds.remove(&label);
        }
    });
}

#[cfg(all(test, target_os = "linux"))]
mod tests {
    use gtk::prelude::*;

    #[test]
    fn test_overlay_reparenting_logic() {
        if gtk::init().is_err() {
            return;
        }

        let window = gtk::Window::new(gtk::WindowType::Toplevel);
        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);
        window.add(&vbox);

        let main_label = gtk::Label::new(Some("main-ui"));
        vbox.pack_start(&main_label, true, true, 0);

        window.remove(&vbox);
        let overlay = gtk::Overlay::new();
        window.add(&overlay);
        overlay.add(&vbox);

        let child_label = gtk::Label::new(Some("child-webview"));
        vbox.pack_start(&child_label, true, true, 0);
        assert_eq!(vbox.children().len(), 2);

        vbox.remove(&child_label);
        overlay.add_overlay(&child_label);
        overlay.set_overlay_pass_through(&child_label, false);

        assert_eq!(vbox.children().len(), 1);
        assert_eq!(vbox.children()[0], main_label.upcast::<gtk::Widget>());
    }
}
