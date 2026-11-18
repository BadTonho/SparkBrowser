//! Smooth movement of page panes when the layout changes (a split is created, changed or closed).
//!
//! Pages are native views, so they cannot be animated with CSS: the new bounds are reached in small
//! steps from a short-lived thread. The curve and the duration match the tab slide in the tab bar.
//! Only layout changes are animated. A window that is being resized, a divider that is being dragged
//! and a page that appears for the first time keep moving at once, because delaying those would
//! make the page lag behind the pointer or the window edge.
use crate::state::browser_state::ViewportBounds;
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicU64, Ordering},
        Mutex, OnceLock,
    },
    time::{Duration, Instant},
};
use tauri::{AppHandle, Manager};
#[cfg(not(target_os = "linux"))]
use tauri::{LogicalPosition, LogicalSize, Rect};

const DURATION: Duration = Duration::from_millis(220);
const FRAME: Duration = Duration::from_millis(16);
/// A change smaller than this (in logical pixels, summed over position and size) is a drag or a
/// rounding difference, not a layout change.
const MIN_CHANGE: f64 = 48.0;
/// Changes that follow each other faster than this belong to a drag and are not animated.
const DRAG_INTERVAL: Duration = Duration::from_millis(140);

#[derive(Default)]
struct Pane {
    /// Where the pane is now (the animation updates it every frame).
    current: Option<ViewportBounds>,
    generation: u64,
    last_change: Option<Instant>,
    /// Where the running slide ends, and when it is over.
    target: Option<ViewportBounds>,
    slide_ends: Option<Instant>,
}

#[derive(Default)]
struct Registry {
    panes: HashMap<String, Pane>,
    /// Size of each window's page area at the last update, to tell a resize from a layout change.
    areas: HashMap<String, (f64, f64)>,
}

static REGISTRY: OnceLock<Mutex<Registry>> = OnceLock::new();
static NEXT_GENERATION: AtomicU64 = AtomicU64::new(1);

fn registry() -> &'static Mutex<Registry> {
    REGISTRY.get_or_init(Mutex::default)
}

/// Easing shared with the tab slide: `cubic-bezier(0.22, 1, 0.36, 1)` is "ease out quint".
fn ease(t: f64) -> f64 {
    1.0 - (1.0 - t.clamp(0.0, 1.0)).powi(5)
}

fn mix(from: ViewportBounds, to: ViewportBounds, t: f64) -> ViewportBounds {
    let lerp = |a: f64, b: f64| a + (b - a) * t;
    ViewportBounds {
        x: lerp(from.x, to.x),
        y: lerp(from.y, to.y),
        width: lerp(from.width, to.width),
        height: lerp(from.height, to.height),
    }
}

fn change(from: ViewportBounds, to: ViewportBounds) -> f64 {
    (from.x - to.x).abs()
        + (from.y - to.y).abs()
        + (from.width - to.width).abs()
        + (from.height - to.height).abs()
}

fn apply(app: &AppHandle, label: &str, bounds: ViewportBounds) {
    #[cfg(target_os = "linux")]
    crate::error_log::ignored(super::linux_layout::update_bounds(
        app,
        label,
        bounds.x as i32,
        bounds.y as i32,
        bounds.width.max(100.0) as i32,
        bounds.height.max(100.0) as i32,
        true,
    ));
    #[cfg(not(target_os = "linux"))]
    if let Some(view) = app.get_webview(label) {
        crate::error_log::ignored(view.set_bounds(Rect {
            position: LogicalPosition::new(bounds.x, bounds.y).into(),
            size: LogicalSize::new(bounds.width.max(100.0), bounds.height.max(100.0)).into(),
        }));
    }
}

/// What to do with a pane whose target bounds are `to`.
pub enum Plan {
    /// Set the bounds now.
    Now,
    /// An animation was started; the caller must not set the bounds.
    Animated,
}

/// Decides how a pane reaches `to` and, when it should slide, starts the slide.
/// `area` is the size of the window's page area, `window_id` the window that owns the pane.
pub fn plan(app: &AppHandle, label: &str, window_id: &str, area: (f64, f64), to: ViewportBounds) -> Plan {
    let now = Instant::now();
    let Ok(mut registry) = registry().lock() else {
        return Plan::Now;
    };
    let resized = registry
        .areas
        .insert(window_id.to_owned(), area)
        .is_none_or(|before| (before.0 - area.0).abs() > 0.5 || (before.1 - area.1).abs() > 0.5);
    let pane = registry.panes.entry(label.to_owned()).or_default();
    let sliding = pane.slide_ends.is_some_and(|end| now < end);
    // The interface reports the same geometry again while a slide runs (it measures the page area
    // after every layout change); that must neither restart nor cut the slide short.
    if sliding && !resized && pane.target.is_some_and(|target| change(target, to) < 0.5) {
        return Plan::Animated;
    }
    // Faster than a person changes layouts: a divider or a window edge being dragged.
    let quick = !sliding
        && pane
            .last_change
            .is_some_and(|at| now.duration_since(at) < DRAG_INTERVAL);
    let from = pane.current;
    pane.last_change = Some(now);
    let animate = match from {
        Some(from) => !resized && !quick && change(from, to) >= MIN_CHANGE,
        None => false,
    };
    pane.generation = NEXT_GENERATION.fetch_add(1, Ordering::Relaxed);
    let generation = pane.generation;
    pane.target = Some(to);
    pane.slide_ends = animate.then(|| now + DURATION + FRAME * 2);
    if !animate {
        pane.current = Some(to);
        return Plan::Now;
    }
    let from = from.expect("checked above");
    drop(registry);
    let thread_app = app.clone();
    let thread_label = label.to_owned();
    let started = std::thread::Builder::new()
        .name("pane-animation".into())
        .spawn(move || run(&thread_app, &thread_label, generation, from, to));
    if started.is_err() {
        // Without a thread the pane simply moves at once.
        if let Ok(mut registry) = self::registry().lock() {
            if let Some(pane) = registry.panes.get_mut(label) {
                pane.current = Some(to);
            }
        }
        return Plan::Now;
    }
    Plan::Animated
}

fn run(app: &AppHandle, label: &str, generation: u64, from: ViewportBounds, to: ViewportBounds) {
    let start = Instant::now();
    loop {
        let t = (start.elapsed().as_secs_f64() / DURATION.as_secs_f64()).min(1.0);
        let bounds = if t >= 1.0 { to } else { mix(from, to, ease(t)) };
        {
            let Ok(mut registry) = registry().lock() else {
                return;
            };
            let Some(pane) = registry.panes.get_mut(label) else {
                return;
            };
            // A newer change (another animation or an immediate set) owns the pane now.
            if pane.generation != generation {
                return;
            }
            pane.current = Some(bounds);
        }
        let handle = app.clone();
        let target = label.to_owned();
        if app
            .run_on_main_thread(move || apply(&handle, &target, bounds))
            .is_err()
            || t >= 1.0
        {
            return;
        }
        std::thread::sleep(FRAME);
    }
}

/// A pane that is hidden or closed has no position to slide from the next time it shows.
pub fn forget(label: &str) {
    if let Ok(mut registry) = registry().lock() {
        registry.panes.remove(label);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bounds(x: f64, width: f64) -> ViewportBounds {
        ViewportBounds { x, y: 80.0, width, height: 500.0 }
    }

    #[test]
    fn the_curve_starts_fast_ends_exactly_and_never_overshoots() {
        assert_eq!(ease(0.0), 0.0);
        assert_eq!(ease(1.0), 1.0);
        assert!(ease(0.2) > 0.6, "ease out: most of the move happens early");
        let mut last = 0.0;
        for step in 1..=100 {
            let value = ease(step as f64 / 100.0);
            assert!(value >= last && value <= 1.0);
            last = value;
        }
    }

    #[test]
    fn halfway_is_between_the_two_layouts_and_the_end_is_exact() {
        let (a, b) = (bounds(0.0, 1000.0), bounds(500.0, 500.0));
        let half = mix(a, b, 0.5);
        assert_eq!((half.x, half.width), (250.0, 750.0));
        let end = mix(a, b, 1.0);
        assert_eq!((end.x, end.width), (500.0, 500.0));
    }

    #[test]
    fn a_repeated_report_of_the_same_layout_does_not_cut_the_slide_short() {
        // The decision logic needs an app to start a thread, so the registry is checked directly.
        let mut registry = Registry::default();
        let pane = registry.panes.entry("page".into()).or_default();
        let now = Instant::now();
        pane.target = Some(bounds(500.0, 500.0));
        pane.slide_ends = Some(now + DURATION);
        let sliding = pane.slide_ends.is_some_and(|end| now < end);
        assert!(sliding && change(pane.target.unwrap(), bounds(500.0, 500.0)) < 0.5);
        assert!(change(pane.target.unwrap(), bounds(0.0, 1000.0)) >= 0.5, "a different layout is a new slide");
    }

    #[test]
    fn only_real_layout_changes_count() {
        assert!(change(bounds(0.0, 1000.0), bounds(500.0, 500.0)) >= MIN_CHANGE);
        assert!(change(bounds(0.0, 1000.0), bounds(0.0, 1004.0)) < MIN_CHANGE);
    }
}
