use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use serde::Serialize;
use tauri::{
    AppHandle, Emitter, LogicalSize, Manager, PhysicalPosition, PhysicalSize, Size, State,
};

use crate::state::AppState;

const SNAP_DISTANCE: f64 = 28.0;
const STRIP_WIDTH: f64 = 44.0;
const MIN_WIDTH: f64 = 260.0;
const MIN_HEIGHT: f64 = 120.0;

#[derive(Serialize, Clone, Copy, PartialEq, Eq, Default, Debug)]
#[serde(rename_all = "lowercase")]
pub enum Edge {
    #[default]
    None,
    Left,
    Right,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DockState {
    pub edge: Edge,
    pub collapsed: bool,
}

#[derive(Default)]
pub struct Dock {
    edge: Mutex<Edge>,
    generation: AtomicU64,
}

pub fn on_moved(app: &AppHandle) {
    let Some(dock) = app.try_state::<Dock>() else {
        return;
    };
    let generation = dock.generation.fetch_add(1, Ordering::SeqCst) + 1;
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(Duration::from_millis(350)).await;
        if app.state::<Dock>().generation.load(Ordering::SeqCst) == generation {
            snap(&app);
        }
    });
}

pub fn snap(app: &AppHandle) {
    let Some(window) = app.get_webview_window("main") else {
        return;
    };
    let (Ok(pos), Ok(size), Ok(Some(monitor))) = (
        window.outer_position(),
        window.outer_size(),
        window.current_monitor(),
    ) else {
        return;
    };

    let area = monitor.work_area();
    let threshold = (SNAP_DISTANCE * monitor.scale_factor()) as i32;
    let (left, top) = (area.position.x, area.position.y);
    let right = left + area.size.width as i32;
    let bottom = top + area.size.height as i32;
    let (width, height) = (size.width as i32, size.height as i32);

    let (mut x, mut y) = (pos.x, pos.y);
    let edge = if pos.x - left <= threshold {
        x = left;
        Edge::Left
    } else if right - (pos.x + width) <= threshold {
        x = right - width;
        Edge::Right
    } else {
        Edge::None
    };
    if y - top <= threshold {
        y = top;
    } else if bottom - (y + height) <= threshold {
        y = bottom - height;
    }

    if (x, y) != (pos.x, pos.y) {
        let _ = window.set_position(PhysicalPosition::new(x, y));
    }
    set_edge(app, edge);
}

fn set_edge(app: &AppHandle, edge: Edge) {
    let dock = app.state::<Dock>();
    let changed = {
        let mut current = dock.edge.lock().unwrap();
        let changed = *current != edge;
        *current = edge;
        changed
    };
    if changed {
        let _ = app.emit("dock-changed", current_state(app));
    }
}

fn current_state(app: &AppHandle) -> DockState {
    DockState {
        edge: *app.state::<Dock>().edge.lock().unwrap(),
        collapsed: app
            .state::<AppState>()
            .store
            .lock()
            .unwrap()
            .settings
            .collapsed,
    }
}

#[tauri::command]
pub fn get_dock(app: AppHandle) -> DockState {
    current_state(&app)
}

#[tauri::command]
pub fn set_collapsed(
    app: AppHandle,
    state: State<'_, AppState>,
    collapsed: bool,
    strip_height: f64,
) -> Result<DockState, String> {
    let window = app
        .get_webview_window("main")
        .ok_or("janela principal não encontrada")?;
    let err = |e: tauri::Error| e.to_string();
    let scale = window.scale_factor().map_err(err)?;
    let old_size = window.outer_size().map_err(err)?;
    let target = {
        let mut s = state.store.lock().unwrap();
        let ui_scale = s.settings.scale();
        if collapsed && !s.settings.collapsed {
            let logical = old_size.to_logical::<f64>(scale);
            s.settings.expanded_width = logical.width / ui_scale;
            s.settings.expanded_height = logical.height / ui_scale;
        }
        s.settings.collapsed = collapsed;
        s.save_settings()?;
        if collapsed {
            LogicalSize::new(STRIP_WIDTH * ui_scale, strip_height.max(64.0) * ui_scale)
        } else {
            LogicalSize::new(
                s.settings.expanded_width.max(MIN_WIDTH) * ui_scale,
                s.settings.expanded_height.max(MIN_HEIGHT) * ui_scale,
            )
        }
    };

    resize(app.clone(), target, collapsed)?;
    let dock = current_state(&app);
    let _ = app.emit("dock-changed", dock.clone());
    Ok(dock)
}

pub fn apply_scale(app: &AppHandle, previous: f64, next: f64, collapsed: bool) -> Result<(), String> {
    if previous == next {
        return Ok(());
    }
    let window = app
        .get_webview_window("main")
        .ok_or("janela principal não encontrada")?;
    let dpi = window.scale_factor().map_err(|e| e.to_string())?;
    let size = window
        .inner_size()
        .map_err(|e| e.to_string())?
        .to_logical::<f64>(dpi);
    resize(
        app.clone(),
        scaled_size(size, previous, next, collapsed),
        collapsed,
    )
}

fn scaled_size(
    size: LogicalSize<f64>,
    previous: f64,
    next: f64,
    collapsed: bool,
) -> LogicalSize<f64> {
    if collapsed {
        LogicalSize::new(STRIP_WIDTH * next, size.height / previous * next)
    } else {
        LogicalSize::new(
            (size.width / previous).max(MIN_WIDTH) * next,
            (size.height / previous).max(MIN_HEIGHT) * next,
        )
    }
}

pub fn initialize(app: &AppHandle) -> Result<(), String> {
    let state = app.state::<AppState>();
    let settings = state.store.lock().unwrap().settings.clone();
    let window = app
        .get_webview_window("main")
        .ok_or("janela principal não encontrada")?;
    let dpi = window.scale_factor().map_err(|e| e.to_string())?;
    let size = window
        .inner_size()
        .map_err(|e| e.to_string())?
        .to_logical::<f64>(dpi);
    let target = if settings.collapsed {
        LogicalSize::new(STRIP_WIDTH * settings.scale(), size.height)
    } else {
        LogicalSize::new(
            size.width.max(MIN_WIDTH * settings.scale()),
            size.height.max(MIN_HEIGHT * settings.scale()),
        )
    };
    resize(app.clone(), target, settings.collapsed)
}

fn resize(app: AppHandle, mut target: LogicalSize<f64>, collapsed: bool) -> Result<(), String> {
    let window = app
        .get_webview_window("main")
        .ok_or("janela principal não encontrada")?;
    let err = |e: tauri::Error| e.to_string();
    let scale = window.scale_factor().map_err(err)?;
    let old_size = window.outer_size().map_err(err)?;
    let pos = window.outer_position().map_err(err)?;
    let ui_scale = app
        .state::<AppState>()
        .store
        .lock()
        .unwrap()
        .settings
        .scale();
    if let Ok(Some(monitor)) = window.current_monitor() {
        target.width = target
            .width
            .min(monitor.work_area().size.width as f64 / scale);
        target.height = target
            .height
            .min(monitor.work_area().size.height as f64 / scale);
    }
    window.set_min_size(None::<Size>).map_err(err)?;
    window.set_resizable(!collapsed).map_err(err)?;
    window.set_size(target).map_err(err)?;
    if !collapsed {
        window
            .set_min_size(Some(LogicalSize::new(MIN_WIDTH * ui_scale, MIN_HEIGHT * ui_scale)))
            .map_err(err)?;
    }

    let new_size: PhysicalSize<u32> = target.to_physical(scale);
    let edge = *app.state::<Dock>().edge.lock().unwrap();
    let mut x = match edge {
        Edge::Right => pos.x + old_size.width as i32 - new_size.width as i32,
        _ => pos.x,
    };
    let mut y = pos.y;
    if let Ok(Some(monitor)) = window.current_monitor() {
        let area = monitor.work_area();
        let right = area.position.x + area.size.width as i32;
        let bottom = area.position.y + area.size.height as i32;
        if (bottom - (pos.y + old_size.height as i32)).abs() <= (SNAP_DISTANCE * scale) as i32 {
            y = pos.y + old_size.height as i32 - new_size.height as i32;
        }
        x = x.clamp(
            area.position.x,
            (right - new_size.width as i32).max(area.position.x),
        );
        y = y.clamp(
            area.position.y,
            (bottom - new_size.height as i32).max(area.position.y),
        );
    }
    window
        .set_position(PhysicalPosition::new(x, y))
        .map_err(err)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repeated_scale_changes_do_not_compound() {
        let original = LogicalSize::new(340.0, 420.0);
        let small = scaled_size(original, 1.0, 0.7, false);
        let medium = scaled_size(small, 0.7, 0.85, false);
        let restored = scaled_size(medium, 0.85, 1.0, false);
        assert!((restored.width - original.width).abs() < 0.001);
        assert!((restored.height - original.height).abs() < 0.001);
    }

    #[test]
    fn strip_and_minimum_expanded_size_follow_scale() {
        let strip = scaled_size(LogicalSize::new(44.0, 206.0), 1.0, 0.85, true);
        assert!((strip.width - 37.4).abs() < 0.001);
        assert!((strip.height - 175.1).abs() < 0.001);
        let expanded = scaled_size(LogicalSize::new(200.0, 100.0), 1.0, 0.7, false);
        assert_eq!(expanded, LogicalSize::new(182.0, 84.0));
    }
}
