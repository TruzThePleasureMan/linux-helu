use gtk4 as gtk;
use gtk::prelude::*;
use std::rc::Rc;
use std::cell::RefCell;
use crate::ring::PulsingRing;
use crate::pinpad::PinPad;

pub struct AuthOverlay {
    root: gtk::Overlay,
    ring: Rc<RefCell<PulsingRing>>,
    status_label: gtk::Label,
    revealer: gtk::Revealer,
    pinpad: PinPad,
}

impl Default for AuthOverlay {
    fn default() -> Self {
        Self::new()
    }
}

impl AuthOverlay {
    pub fn new() -> Self {
        let root = gtk::Overlay::new();

        let box_widget = gtk::Box::new(gtk::Orientation::Vertical, 16);
        box_widget.set_halign(gtk::Align::Center);
        box_widget.set_valign(gtk::Align::Center);
        box_widget.set_width_request(400);

        let ring = Rc::new(RefCell::new(PulsingRing::new()));
        box_widget.append(ring.borrow().widget());

        let greeting = gtk::Label::new(Some("Helu, User"));
        greeting.add_css_class("helu-greeting");
        box_widget.append(&greeting);

        let status_label = gtk::Label::new(Some("Looking for you..."));
        status_label.add_css_class("helu-status");
        box_widget.append(&status_label);

        let revealer = gtk::Revealer::new();
        revealer.set_transition_type(gtk::RevealerTransitionType::SlideUp);

        let pinpad = PinPad::new();
        revealer.set_child(Some(&pinpad.widget()));
        box_widget.append(&revealer);

        root.set_child(Some(&box_widget));

        Self {
            root,
            ring,
            status_label,
            revealer,
            pinpad,
        }
    }

    pub fn widget(&self) -> &gtk::Widget {
        self.root.upcast_ref()
    }

    pub fn handle_event(&self, event: crate::dbus::UiEvent) {
        match event {
            crate::dbus::UiEvent::StateChange(state) => {
                match state.state.as_str() {
                    "Idle" => {
                        self.status_label.set_text("Waiting...");
                        self.ring.borrow_mut().set_state(crate::ring::RingState::Idle);
                        self.revealer.set_reveal_child(false);
                    }
                    "Scanning" => {
                        self.status_label.set_text("Looking for you...");
                        self.ring.borrow_mut().set_state(crate::ring::RingState::Scanning);
                        self.revealer.set_reveal_child(false);
                    }
                    "Success" => {
                        self.status_label.set_text("Welcome back");
                        self.ring.borrow_mut().set_state(crate::ring::RingState::Success);
                        self.revealer.set_reveal_child(false);
                        // Hide after 1.5s? For now, handled by daemon or next state.
                    }
                    "Failure" => {
                        let msg = format!("{} (Retries: {})", state.reason, state.retry_count);
                        self.status_label.set_text(&msg);
                        self.ring.borrow_mut().set_state(crate::ring::RingState::Failure);
                    }
                    "Fallback" | "FidoPrompt" => {
                        self.status_label.set_text("Enter PIN");
                        self.ring.borrow_mut().set_state(crate::ring::RingState::Idle);
                        self.revealer.set_reveal_child(true);
                        self.pinpad.focus();
                    }
                    _ => {}
                }
            }
        }
    }
}
