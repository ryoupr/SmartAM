use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use tauri::{
    menu::{MenuBuilder, MenuItemBuilder},
    tray::TrayIconBuilder,
    App, AppHandle, Manager,
};

static UNREAD_COUNT: AtomicUsize = AtomicUsize::new(0);
static PAUSED: AtomicBool = AtomicBool::new(false);

const ID_SHOW: &str = "show_window";
const ID_BADGE: &str = "badge";
const ID_PAUSE: &str = "pause";
const ID_QUIT: &str = "quit";

pub fn setup(app: &App) -> tauri::Result<()> {
    let menu = MenuBuilder::new(app)
        .item(&MenuItemBuilder::with_id(ID_SHOW, "ウィンドウを表示").build(app)?)
        .separator()
        .item(&MenuItemBuilder::with_id(ID_BADGE, "新着 0 件").enabled(false).build(app)?)
        .item(&MenuItemBuilder::with_id(ID_PAUSE, "通知を一時停止").build(app)?)
        .separator()
        .item(&MenuItemBuilder::with_id(ID_QUIT, "終了").build(app)?)
        .build()?;

    TrayIconBuilder::new()
        .icon(app.default_window_icon().cloned().unwrap())
        .icon_as_template(true)
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_menu_event(|app, event| {
            match event.id().as_ref() {
                x if x == ID_SHOW => show_window(app),
                x if x == ID_PAUSE => toggle_pause(app),
                x if x == ID_QUIT => app.exit(0),
                _ => {}
            }
        })
        .build(app)?;

    Ok(())
}

pub fn update_badge(app: &AppHandle, count: usize) {
    UNREAD_COUNT.store(count, Ordering::Relaxed);
    let label = format!("新着 {} 件", count);
    // Update menu item text
    if let Some(tray) = app.tray_by_id("main") {
        let _ = tray.set_title(Some(&format!("{}", if count > 0 { count.to_string() } else { String::new() })));
    }
    // Update via menu item
    update_badge_menu_item(app, &label);
}

pub fn toggle_pause(app: &AppHandle) {
    let was_paused = PAUSED.fetch_xor(true, Ordering::Relaxed);
    let now_paused = !was_paused;
    let label = if now_paused { "通知を再開" } else { "通知を一時停止" };
    update_pause_menu_item(app, label);
    // Sync with IdleWatcher
    tauri::async_runtime::spawn(async move {
        crate::idle_watcher::set_paused(now_paused).await;
    });
    log::debug!("tray: notification pause = {}", now_paused);
}

fn show_window(app: &AppHandle) {
    log::debug!("tray: show_window called");
    if let Some(window) = app.get_webview_window("main") {
        log::debug!("tray: found main window, showing");
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    } else {
        log::error!("tray: main window not found");
    }
}

fn update_badge_menu_item(app: &AppHandle, label: &str) {
    // Tauri v2 doesn't have direct menu item text update by ID on tray,
    // so we rebuild isn't needed — the menu is static for now.
    // Future: use app.menu_by_id() when available
    let _ = (app, label); // suppress unused
}

fn update_pause_menu_item(app: &AppHandle, _label: &str) {
    let _ = app; // Future: dynamic menu update
}
