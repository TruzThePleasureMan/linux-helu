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
            std::thread::spawn(move || {
                let states = vec![
                    helu_common::types::AuthState { state: "Idle".into(), reason: "".into(), retry_count: 0 },
                    helu_common::types::AuthState { state: "Scanning".into(), reason: "".into(), retry_count: 0 },
                    helu_common::types::AuthState { state: "Failure".into(), reason: "Face not recognized".into(), retry_count: 1 },
                    helu_common::types::AuthState { state: "Fallback".into(), reason: "".into(), retry_count: 0 },
                ];
                for state in states.into_iter().cycle() {
                    std::thread::sleep(std::time::Duration::from_secs(3));
                    let _ = tx.send_blocking(dbus::UiEvent::StateChange(state));
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
