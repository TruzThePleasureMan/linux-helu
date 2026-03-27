use gtk4 as gtk;
use gtk::prelude::*;
use libadwaita as adw;
use gtk::glib;

pub mod window;
pub mod overlay;
pub mod ring;
pub mod pinpad;
pub mod dbus;

#[tokio::main]
async fn main() {
    let mock_mode = std::env::args().any(|arg| arg == "--mock");

    // Initialize libadwaita
    adw::init().expect("Failed to initialize libadwaita");

    let app = adw::Application::builder()
        .application_id("net.helu.UI")
        .build();

    app.connect_startup(|_| {
        // Load custom CSS
        let provider = gtk::CssProvider::new();
        provider.load_from_string(include_str!("style.css"));
        gtk::style_context_add_provider_for_display(
            &gtk::gdk::Display::default().expect("Could not connect to a display"),
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    });

    app.connect_activate(move |app| {
        let (tx, rx) = async_channel::unbounded::<dbus::UiEvent>();
        let ui_window = window::build(app, tx.clone());

        if mock_mode {
            println!("Starting in mock mode...");
            // Spawns a background task that ticks states into the UI
            // Cycle requested: Idle (1s) → Scanning (2s) → Failure/attempt 1 (1.5s) →
            // Failure/attempt 2 (1.5s) → PinFallback (2s) → Success (1.5s) → Idle
            std::thread::spawn(move || {
                loop {
                    let _ = tx.send_blocking(dbus::UiEvent::StateChange(helu_common::types::AuthState { username: "user".into(), state: "Idle".into(), reason: "".into(), retry_count: 0 }));
                    std::thread::sleep(std::time::Duration::from_millis(1000));

                    let _ = tx.send_blocking(dbus::UiEvent::AuthRequested { username: "user".into() });
                    std::thread::sleep(std::time::Duration::from_millis(2000));

                    let _ = tx.send_blocking(dbus::UiEvent::AuthFailure { username: "user".into(), reason: "Face not recognized".into() });
                    std::thread::sleep(std::time::Duration::from_millis(1500));

                    let _ = tx.send_blocking(dbus::UiEvent::AuthFailure { username: "user".into(), reason: "Face not recognized".into() });
                    std::thread::sleep(std::time::Duration::from_millis(1500));

                    let _ = tx.send_blocking(dbus::UiEvent::AuthFailure { username: "user".into(), reason: "Face not recognized".into() }); // trigger PinFallback
                    std::thread::sleep(std::time::Duration::from_millis(2000));

                    let _ = tx.send_blocking(dbus::UiEvent::AuthSuccess { username: "user".into() });
                    std::thread::sleep(std::time::Duration::from_millis(1500));
                }
            });
        } else {
            // Setup real DBus in the background
            let tx_clone = tx.clone();
            tokio::spawn(async move {
                if let Err(e) = dbus::listen_to_daemon(tx_clone).await {
                    eprintln!("D-Bus error: {:?}", e);
                }
            });
        }

        // Listen to UI events (from dbus or mock)
        let ctx = glib::MainContext::default();
        ctx.spawn_local(glib::clone!(
            #[strong] rx,
            async move {
                while let Ok(event) = rx.recv().await {
                    ui_window.handle_event(event);
                }
            }
        ));
    });

    // Run the app, ignore command line arguments because we parsed --mock
    app.run_with_args(&[] as &[&str]);
}
