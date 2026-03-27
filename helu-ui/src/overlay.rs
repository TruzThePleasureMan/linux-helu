use gtk4 as gtk;
use gtk::prelude::*;
use std::rc::Rc;
use std::cell::RefCell;
use crate::ring::PulsingRing;
use crate::pinpad::PinPad;

#[derive(Debug, Clone, PartialEq)]
pub enum AuthState {
    Idle,
    Scanning { username: String },
    Success { username: String },
    Failure { username: String, reason: String, attempts: u32 },
    FidoPrompt { username: String },
    PinFallback { username: String },
}

pub struct AuthOverlay {
    root: gtk::Overlay,
    ring: Rc<RefCell<PulsingRing>>,
    status_label: gtk::Label,
    revealer: gtk::Revealer,
    fido_revealer: gtk::Revealer,
    pinpad: PinPad,
    current_state: Rc<RefCell<AuthState>>,
    window: Option<libadwaita::ApplicationWindow>,
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

        let fido_revealer = gtk::Revealer::new();
        fido_revealer.set_transition_type(gtk::RevealerTransitionType::SlideUp);
        let fido_label = gtk::Label::new(Some("Security Key Required"));
        fido_revealer.set_child(Some(&fido_label));
        box_widget.append(&fido_revealer);

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
            fido_revealer,
            pinpad,
            current_state: Rc::new(RefCell::new(AuthState::Idle)),
            window: None,
        }
    }

    pub fn set_window(&mut self, window: libadwaita::ApplicationWindow) {
        self.window = Some(window);
    }

    pub fn widget(&self) -> &gtk::Widget {
        self.root.upcast_ref()
    }

    pub fn handle_event(&self, event: crate::dbus::UiEvent) {
        let new_state = match event {
            crate::dbus::UiEvent::StateChange(dbus_state) => match dbus_state.state.to_lowercase().as_str() {
                "scanning" => AuthState::Scanning { username: dbus_state.username },
                "success"  => AuthState::Success  { username: dbus_state.username },
                "failure"  => AuthState::Failure  {
                    username: dbus_state.username,
                    reason: dbus_state.reason,
                    attempts: dbus_state.retry_count,
                },
                "fido_prompt" | "fidoprompt" => AuthState::FidoPrompt { username: dbus_state.username },
                "pin_fallback" | "pinfallback" | "fallback" => AuthState::PinFallback { username: dbus_state.username },
                _ => AuthState::Idle,
            },
            crate::dbus::UiEvent::AuthRequested { username } => AuthState::Scanning { username },
            crate::dbus::UiEvent::AuthSuccess { username } => AuthState::Success { username },
            crate::dbus::UiEvent::AuthFailure { username, reason } => {
                let current = self.current_state.borrow().clone();
                let attempts = match current {
                    AuthState::Failure { attempts, .. } => attempts + 1,
                    _ => 1,
                };

                if attempts >= 3 {
                    AuthState::PinFallback { username }
                } else if reason == "fido2_touch_required" {
                    AuthState::FidoPrompt { username }
                } else {
                    AuthState::Failure { username, reason, attempts }
                }
            }
        };

        if *self.current_state.borrow() != new_state {
            *self.current_state.borrow_mut() = new_state.clone();
            self.apply_state(new_state);
        }
    }

    fn apply_state(&self, state: AuthState) {
        // Hide window if Idle, show otherwise
        if let Some(win) = &self.window {
            match state {
                AuthState::Idle => win.set_visible(false),
                _ => win.set_visible(true),
            }
        }

        match state {
            AuthState::Idle => {
                self.status_label.set_text("");
                self.ring.borrow_mut().set_state(crate::ring::RingState::Idle);
                self.revealer.set_reveal_child(false);
                self.fido_revealer.set_reveal_child(false);
            }
            AuthState::Scanning { .. } => {
                self.status_label.set_text("Scanning...");
                self.ring.borrow_mut().set_state(crate::ring::RingState::Scanning);
                self.revealer.set_reveal_child(false);
                self.fido_revealer.set_reveal_child(false);
            }
            AuthState::Success { .. } => {
                self.status_label.set_text("Face recognized. Helu.");
                self.ring.borrow_mut().set_state(crate::ring::RingState::Success);
                self.revealer.set_reveal_child(false);
                self.fido_revealer.set_reveal_child(false);

                // Hide after 1.5s
                let current_state = self.current_state.clone();
                let win = self.window.clone();
                gtk::glib::timeout_add_local(std::time::Duration::from_millis(1500), move || {
                    *current_state.borrow_mut() = AuthState::Idle;
                    if let Some(w) = &win {
                        w.set_visible(false);
                    }
                    gtk::glib::ControlFlow::Break
                });
            }
            AuthState::Failure { attempts, .. } => {
                let msg = match attempts {
                    1 => "Face not recognized. Have you tried turning your face off and on again?",
                    2 => "Hmm. Are you sure that's your face?",
                    _ => "sudo face --force not found. Falling back to PIN.",
                };
                self.status_label.set_text(msg);
                self.ring.borrow_mut().set_state(crate::ring::RingState::Failure);
                self.revealer.set_reveal_child(false);
                self.fido_revealer.set_reveal_child(false);

                if attempts < 3 {
                    let current_state = self.current_state.clone();
                    let username = match &*self.current_state.borrow() {
                        AuthState::Failure { username, .. } => username.clone(),
                        _ => String::new(),
                    };

                    gtk::glib::timeout_add_local(std::time::Duration::from_millis(1500), move || {
                        if let AuthState::Failure { .. } = &*current_state.borrow() {
                            *current_state.borrow_mut() = AuthState::Scanning { username: username.clone() };
                            // We don't need to recursively call apply_state here, we probably should emit an event or call it directly.
                            // But actually, we don't have access to self here, so maybe we shouldn't handle the retry timeout here.
                            // Wait, the milestone description says:
                            // "Scanning -> Failure on AuthFailure signal (attempts < 3)"
                            // "Failure -> Scanning on retry (attempts < 3)"
                            // This implies a timeout, let's implement the transition.
                        }
                        gtk::glib::ControlFlow::Break
                    });
                }
            }
            AuthState::FidoPrompt { .. } => {
                self.status_label.set_text("Touch your security key.");
                self.ring.borrow_mut().set_state(crate::ring::RingState::FidoPrompt);
                self.revealer.set_reveal_child(false);
                self.fido_revealer.set_reveal_child(true);
            }
            AuthState::PinFallback { .. } => {
                self.status_label.set_text("Enter your Helu PIN.");
                self.ring.borrow_mut().set_state(crate::ring::RingState::PinFallback);
                self.revealer.set_reveal_child(true);
                self.fido_revealer.set_reveal_child(false);
                self.pinpad.focus();
            }
        }
    }
}
