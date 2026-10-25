mod browsers;
mod commands;
mod daemon;
mod dns_guard;
mod tray;
mod watch;

use tauri::{RunEvent, WindowEvent};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            tray::build(app.handle())?;
            watch::watch_daemon();
            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::load_rules,
            commands::save_rules,
            commands::set_password,
            commands::consent_adult_filter,
            commands::request_adult_filter_off,
            commands::cancel_adult_filter_off,
            commands::enable_adult_filter,
            commands::add_recipient,
            commands::request_recipient_removal,
            commands::cancel_recipient_removal,
            commands::set_mail_account,
            commands::recent_events,
            commands::quit_app,
            commands::daemon_status,
            commands::install_daemon,
            commands::uninstall_daemon,
            commands::running_browsers,
            commands::quit_browsers
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|_app, event| {
            if let RunEvent::Exit = event {
                watch::notify_quit_if_blocking();
            }
        });
}
