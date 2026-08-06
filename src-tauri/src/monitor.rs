use std::{sync::Arc, time::Duration};

use tauri::{Emitter, Manager, PhysicalPosition, PhysicalSize};
use windows_sys::Win32::{
    Foundation::{POINT, RECT},
    Graphics::Gdi::{GetMonitorInfoW, MonitorFromPoint, MONITORINFO, MONITOR_DEFAULTTONEAREST},
    UI::WindowsAndMessaging::GetCursorPos,
};

use crate::{
    models::{AppSettings, MonitorInfo, Point, Size},
    state::AppState,
    time::unix_ms,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Bounds {
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
}

impl Bounds {
    fn from_position_size(position: Point, size: Size) -> Self {
        Self {
            left: position.x,
            top: position.y,
            right: position.x.saturating_add(size.width as i32),
            bottom: position.y.saturating_add(size.height as i32),
        }
    }

    fn contains_inset(self, point: Point, inset: i32) -> bool {
        point.x >= self.left.saturating_add(inset)
            && point.x < self.right.saturating_sub(inset)
            && point.y >= self.top.saturating_add(inset)
            && point.y < self.bottom.saturating_sub(inset)
    }

    fn contains_expanded(self, point: Point, expansion: i32) -> bool {
        point.x >= self.left.saturating_sub(expansion)
            && point.x < self.right.saturating_add(expansion)
            && point.y >= self.top.saturating_sub(expansion)
            && point.y < self.bottom.saturating_add(expansion)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OverlayPhase {
    Visible,
    EntryPending(u64),
    Hidden,
    ReturnPending(u64),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OverlayAction {
    None,
    Hide,
    Show,
}

#[derive(Debug, Clone, Copy)]
struct OverlayMachine {
    phase: OverlayPhase,
}

impl Default for OverlayMachine {
    fn default() -> Self {
        Self {
            phase: OverlayPhase::Hidden,
        }
    }
}

impl OverlayMachine {
    fn reset_hidden(&mut self) {
        self.phase = OverlayPhase::Hidden;
    }

    fn should_be_visible(self) -> bool {
        matches!(
            self.phase,
            OverlayPhase::Visible | OverlayPhase::EntryPending(_)
        )
    }

    fn tick(
        &mut self,
        now_ms: u64,
        inside_enter_bounds: bool,
        inside_exit_bounds: bool,
        entry_dwell_ms: u64,
        return_delay_ms: u64,
    ) -> OverlayAction {
        match self.phase {
            OverlayPhase::Visible => {
                if inside_enter_bounds {
                    if entry_dwell_ms == 0 {
                        self.phase = OverlayPhase::Hidden;
                        OverlayAction::Hide
                    } else {
                        self.phase = OverlayPhase::EntryPending(now_ms);
                        OverlayAction::None
                    }
                } else {
                    OverlayAction::None
                }
            }
            OverlayPhase::EntryPending(started) => {
                if !inside_exit_bounds {
                    self.phase = OverlayPhase::Visible;
                    OverlayAction::None
                } else if now_ms.saturating_sub(started) >= entry_dwell_ms {
                    self.phase = OverlayPhase::Hidden;
                    OverlayAction::Hide
                } else {
                    OverlayAction::None
                }
            }
            OverlayPhase::Hidden => {
                if inside_exit_bounds {
                    OverlayAction::None
                } else if return_delay_ms == 0 {
                    self.phase = OverlayPhase::Visible;
                    OverlayAction::Show
                } else {
                    self.phase = OverlayPhase::ReturnPending(now_ms);
                    OverlayAction::None
                }
            }
            OverlayPhase::ReturnPending(started) => {
                if inside_exit_bounds {
                    self.phase = OverlayPhase::Hidden;
                    OverlayAction::None
                } else if now_ms.saturating_sub(started) >= return_delay_ms {
                    self.phase = OverlayPhase::Visible;
                    OverlayAction::Show
                } else {
                    OverlayAction::None
                }
            }
        }
    }
}

fn native_work_area(center: Point) -> Option<(Point, Size)> {
    // SAFETY: MonitorFromPoint has no pointer arguments. MONITORINFO is fully
    // initialized with its required byte size before GetMonitorInfoW writes it.
    unsafe {
        let handle = MonitorFromPoint(
            POINT {
                x: center.x,
                y: center.y,
            },
            MONITOR_DEFAULTTONEAREST,
        );
        if handle.is_null() {
            return None;
        }
        let mut info: MONITORINFO = std::mem::zeroed();
        info.cbSize = std::mem::size_of::<MONITORINFO>() as u32;
        if GetMonitorInfoW(handle, &mut info) == 0 {
            return None;
        }
        Some(rect_to_geometry(info.rcWork))
    }
}

fn rect_to_geometry(rect: RECT) -> (Point, Size) {
    (
        Point {
            x: rect.left,
            y: rect.top,
        },
        Size {
            width: rect.right.saturating_sub(rect.left) as u32,
            height: rect.bottom.saturating_sub(rect.top) as u32,
        },
    )
}

pub fn list(app: &tauri::AppHandle) -> Result<Vec<MonitorInfo>, String> {
    let window = app
        .get_webview_window("settings")
        .or_else(|| app.get_webview_window("display"))
        .ok_or_else(|| "No application window is available".to_owned())?;
    let monitors = window
        .available_monitors()
        .map_err(|error| format!("Could not enumerate displays: {error}"))?;
    let primary = window.primary_monitor().ok().flatten();

    Ok(monitors
        .into_iter()
        .enumerate()
        .map(|(index, monitor)| {
            let native_name = monitor
                .name()
                .map(String::as_str)
                .unwrap_or("Unknown display")
                .to_owned();
            let position = Point {
                x: monitor.position().x,
                y: monitor.position().y,
            };
            let size = Size {
                width: monitor.size().width,
                height: monitor.size().height,
            };
            let center = Point {
                x: position.x.saturating_add((size.width / 2) as i32),
                y: position.y.saturating_add((size.height / 2) as i32),
            };
            let (work_area_position, work_area_size) =
                native_work_area(center).unwrap_or((position, size));
            let is_primary = primary.as_ref().is_some_and(|candidate| {
                candidate.position() == monitor.position() && candidate.size() == monitor.size()
            });

            MonitorInfo {
                id: native_name.clone(),
                name: if native_name == "Unknown display" {
                    format!("Display {}", index + 1)
                } else {
                    native_name
                },
                is_primary,
                position,
                size,
                work_area_position,
                work_area_size,
                scale_factor: monitor.scale_factor(),
            }
        })
        .collect())
}

pub fn resolve<'a>(settings: &AppSettings, monitors: &'a [MonitorInfo]) -> Option<&'a MonitorInfo> {
    if let Some(id) = settings.selected_monitor_id.as_deref() {
        if let Some(monitor) = monitors.iter().find(|monitor| monitor.id == id) {
            return Some(monitor);
        }
    }
    if let Some(name) = settings.selected_monitor_name.as_deref() {
        if let Some(monitor) = monitors.iter().find(|monitor| monitor.name == name) {
            return Some(monitor);
        }
    }
    match (
        settings.selected_monitor_position,
        settings.selected_monitor_size,
    ) {
        (Some(position), Some(size)) => monitors
            .iter()
            .min_by_key(|monitor| geometry_distance(position, size, monitor)),
        _ => None,
    }
}

fn geometry_distance(position: Point, size: Size, monitor: &MonitorInfo) -> u64 {
    u64::from(position.x.abs_diff(monitor.position.x))
        + u64::from(position.y.abs_diff(monitor.position.y))
        + u64::from(size.width.abs_diff(monitor.size.width))
        + u64::from(size.height.abs_diff(monitor.size.height))
}

fn cursor_position() -> Option<Point> {
    let mut point = POINT::default();
    // SAFETY: point is a valid writable POINT for the duration of the call.
    if unsafe { GetCursorPos(&mut point) } == 0 {
        None
    } else {
        Some(Point {
            x: point.x,
            y: point.y,
        })
    }
}

fn position_display(app: &tauri::AppHandle, monitor: &MonitorInfo, cover_taskbar: bool) {
    let Some(window) = app.get_webview_window("display") else {
        return;
    };
    let (position, size) = if cover_taskbar {
        (monitor.position, monitor.size)
    } else {
        (monitor.work_area_position, monitor.work_area_size)
    };
    let _ = window.set_position(PhysicalPosition::new(position.x, position.y));
    let _ = window.set_size(PhysicalSize::new(size.width, size.height));
    let _ = window.set_always_on_top(true);
    let _ = window.set_skip_taskbar(true);
    let _ = window.set_focusable(false);
    // The display is intentionally click-through. Native global pointer tracking
    // hides it, and the first click always reaches the desktop beneath it.
    let _ = window.set_ignore_cursor_events(true);
}

pub fn show_display(app: &tauri::AppHandle, monitor: &MonitorInfo, cover_taskbar: bool) {
    position_display(app, monitor, cover_taskbar);
    if let Some(window) = app.get_webview_window("display") {
        let _ = window.show();
    }
}

pub fn hide_display(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("display") {
        let _ = window.hide();
    }
}

fn sync_display_visibility(
    app: &tauri::AppHandle,
    monitor: &MonitorInfo,
    cover_taskbar: bool,
    should_be_visible: bool,
) {
    let is_visible = app
        .get_webview_window("display")
        .and_then(|window| window.is_visible().ok())
        .unwrap_or(false);
    match (should_be_visible, is_visible) {
        (true, false) => show_display(app, monitor, cover_taskbar),
        (false, true) => hide_display(app),
        _ => {}
    }
}

pub fn start_worker(app: tauri::AppHandle, state: Arc<AppState>) {
    tauri::async_runtime::spawn(async move {
        let mut machine = OverlayMachine::default();
        let mut monitors = Vec::new();
        let mut last_monitor_refresh = 0_u64;
        let mut selected_signature: Option<(String, Point, Size, bool)> = None;

        loop {
            let now = unix_ms();
            if now.saturating_sub(last_monitor_refresh) >= 1_000 {
                let previous = monitors.clone();
                monitors = list(&app).unwrap_or_default();
                last_monitor_refresh = now;
                if monitors != previous {
                    let _ = app.emit("monitors://changed", &monitors);
                }
            }

            let settings = state.settings.read().await.clone();
            let Some(monitor) = resolve(&settings, &monitors) else {
                hide_display(&app);
                machine.reset_hidden();
                selected_signature = None;
                tokio::time::sleep(Duration::from_millis(100)).await;
                continue;
            };

            let signature = (
                monitor.id.clone(),
                if settings.cover_taskbar {
                    monitor.position
                } else {
                    monitor.work_area_position
                },
                if settings.cover_taskbar {
                    monitor.size
                } else {
                    monitor.work_area_size
                },
                settings.cover_taskbar,
            );
            if selected_signature.as_ref() != Some(&signature) {
                position_display(&app, monitor, settings.cover_taskbar);
                selected_signature = Some(signature);
            }

            if !settings.setup_complete {
                hide_display(&app);
                machine.reset_hidden();
                tokio::time::sleep(Duration::from_millis(100)).await;
                continue;
            }

            let settings_visible = app
                .get_webview_window("settings")
                .and_then(|window| window.is_visible().ok())
                .unwrap_or(false);
            let display_suppressed = state
                .display_suppressed
                .load(std::sync::atomic::Ordering::Relaxed);
            if settings_visible || display_suppressed {
                sync_display_visibility(&app, monitor, settings.cover_taskbar, false);
                machine.reset_hidden();
                tokio::time::sleep(Duration::from_millis(32)).await;
                continue;
            }

            let preview_until = state
                .preview_until_unix_ms
                .load(std::sync::atomic::Ordering::Relaxed);
            if now < preview_until {
                sync_display_visibility(&app, monitor, settings.cover_taskbar, true);
                tokio::time::sleep(Duration::from_millis(32)).await;
                continue;
            }

            if let Some(cursor) = cursor_position() {
                let bounds = Bounds::from_position_size(monitor.position, monitor.size);
                let hysteresis = settings.edge_hysteresis_px;
                machine.tick(
                    now,
                    // Hide on the first physical pixel of the selected monitor.
                    // Hysteresis is only needed while deciding when it is safe
                    // to return, otherwise a pointer resting at an edge can leave
                    // the display visible over the desktop.
                    bounds.contains_inset(cursor, 0),
                    bounds.contains_expanded(cursor, hysteresis),
                    settings.entry_dwell_ms,
                    settings.return_delay_ms,
                );
                sync_display_visibility(
                    &app,
                    monitor,
                    settings.cover_taskbar,
                    machine.should_be_visible(),
                );
            } else {
                // If Windows temporarily cannot report the global cursor, fail
                // out of the user's way and recover through the normal return
                // debounce once coordinates are available again.
                sync_display_visibility(&app, monitor, settings.cover_taskbar, false);
                machine.reset_hidden();
            }
            tokio::time::sleep(Duration::from_millis(16)).await;
        }
    });
}

#[cfg(test)]
mod tests {
    use super::{Bounds, OverlayAction, OverlayMachine, OverlayPhase};
    use crate::models::Point;

    #[test]
    fn negative_desktop_coordinates_are_inside() {
        let bounds = Bounds {
            left: -1920,
            top: 0,
            right: 0,
            bottom: 1080,
        };
        assert!(bounds.contains_inset(Point { x: -1200, y: 400 }, 4));
        assert!(!bounds.contains_inset(Point { x: 100, y: 400 }, 4));
    }

    #[test]
    fn hides_immediately_on_entry_by_default() {
        let mut machine = OverlayMachine {
            phase: OverlayPhase::Visible,
        };
        assert_eq!(machine.tick(100, true, true, 0, 500), OverlayAction::Hide);
        assert_eq!(machine.phase, OverlayPhase::Hidden);
    }

    #[test]
    fn dwell_cancels_when_pointer_leaves() {
        let mut machine = OverlayMachine {
            phase: OverlayPhase::Visible,
        };
        assert_eq!(machine.tick(100, true, true, 150, 500), OverlayAction::None);
        assert_eq!(
            machine.tick(180, false, false, 150, 500),
            OverlayAction::None
        );
        assert_eq!(machine.phase, OverlayPhase::Visible);
    }

    #[test]
    fn return_delay_requires_continuous_time_outside() {
        let mut machine = OverlayMachine::default();
        assert_eq!(machine.tick(100, false, false, 0, 500), OverlayAction::None);
        assert_eq!(machine.tick(599, false, false, 0, 500), OverlayAction::None);
        assert_eq!(machine.tick(600, false, false, 0, 500), OverlayAction::Show);
    }

    #[test]
    fn edge_hysteresis_uses_inset_and_expansion() {
        let bounds = Bounds {
            left: 0,
            top: 0,
            right: 1920,
            bottom: 1080,
        };
        let noisy_edge = Point { x: 2, y: 500 };
        assert!(bounds.contains_inset(noisy_edge, 0));
        assert!(!bounds.contains_inset(noisy_edge, 4));
        assert!(bounds.contains_expanded(Point { x: -2, y: 500 }, 4));
    }

    #[test]
    fn previewed_window_reconciles_hidden_when_pointer_is_inside() {
        let mut machine = OverlayMachine::default();
        assert_eq!(machine.tick(100, true, true, 0, 500), OverlayAction::None);
        assert!(!machine.should_be_visible());
    }
}
