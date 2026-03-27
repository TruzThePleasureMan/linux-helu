use gtk4 as gtk;
use gtk::prelude::*;
use libadwaita as adw;
use gtk4_layer_shell::{Layer, LayerShell, Edge, KeyboardMode};
use libadwaita::prelude::AdwApplicationWindowExt;

pub fn setup_layer_shell(window: &adw::ApplicationWindow) {
    window.init_layer_shell();
    window.set_layer(Layer::Overlay);
    window.set_exclusive_zone(-1);
    window.set_anchor(Edge::Top, true);
    window.set_anchor(Edge::Bottom, true);
    window.set_anchor(Edge::Left, true);
    window.set_anchor(Edge::Right, true);
    window.set_keyboard_mode(KeyboardMode::Exclusive);
}

pub fn build(app: &adw::Application, _tx: async_channel::Sender<crate::dbus::UiEvent>) -> crate::overlay::AuthOverlay {
    let window = adw::ApplicationWindow::new(app);

    if gtk4_layer_shell::is_supported() {
        setup_layer_shell(&window);
    } else {
        // X11 fallback
        window.set_decorated(false);
    }

    let mut overlay = crate::overlay::AuthOverlay::new();
    overlay.set_window(window.clone());
    window.set_content(Some(overlay.widget()));
    window.set_visible(false);

    // Tell daemon UI is ready via DBus within 500ms
    // We already do this via spawn below, but we can add a slight delay if needed.
    // Spawning it directly should be within 500ms.
    // If connection fails, log and exit with non-zero
    tokio::spawn(async {
        // give glib loop a tiny moment to start
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        if let Err(e) = crate::dbus::emit_ui_ready().await {
            eprintln!("Failed to emit UIReady: {:?}", e);
            std::process::exit(1);
        }
    });

    overlay
}
