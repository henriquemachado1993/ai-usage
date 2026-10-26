use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use serde::Serialize;
use tauri::{
    AppHandle, Emitter, LogicalSize, Manager, PhysicalPosition, PhysicalSize, Size, State,
};

use crate::state::AppState;

const SNAP_DISTANCE: f64 = 28.0;
const STRIP_WIDTH: f64 = 60.0;
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
    let pos = window.outer_position().map_err(err)?;

    let target = {
        let mut s = state.store.lock().unwrap();
        if collapsed && !s.settings.collapsed {
            let logical = old_size.to_logical::<f64>(scale);
            s.settings.expanded_width = logical.width;
            s.settings.expanded_height = logical.height;
        }
        s.settings.collapsed = collapsed;
        s.save_settings()?;
        if collapsed {
            LogicalSize::new(STRIP_WIDTH, strip_height)
        } else {
            LogicalSize::new(
                s.settings.expanded_width.max(MIN_WIDTH),
                s.settings.expanded_height.max(MIN_HEIGHT),
            )
        }
    };

    if collapsed {
        window.set_min_size(None::<Size>).map_err(err)?;
    }
    window.set_resizable(!collapsed).map_err(err)?;
    window.set_size(target).map_err(err)?;
    if !collapsed {
        window
            .set_min_size(Some(LogicalSize::new(MIN_WIDTH, MIN_HEIGHT)))
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

    let dock = current_state(&app);
    let _ = app.emit("dock-changed", dock.clone());
    Ok(dock)
}
