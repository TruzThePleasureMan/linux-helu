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

    let overlay = crate::overlay::AuthOverlay::new();
    window.set_content(Some(overlay.widget()));
    window.present();

    // Tell daemon UI is ready via DBus
    tokio::spawn(async {
        if let Err(e) = crate::dbus::emit_ui_ready().await {
            eprintln!("Failed to emit UIReady: {:?}", e);
        }
    });

    overlay
}
