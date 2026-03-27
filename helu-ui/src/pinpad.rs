use gtk4 as gtk;
use gtk::prelude::*;

pub struct PinPad {
    entry: gtk::PasswordEntry,
    grid: gtk::Grid,
    button: gtk::Button,
}

impl Default for PinPad {
    fn default() -> Self {
        Self::new()
    }
}

impl PinPad {
    pub fn new() -> Self {
        let entry = gtk::PasswordEntry::new();
        entry.set_show_peek_icon(true);
        entry.set_halign(gtk::Align::Center);

        let grid = gtk::Grid::new();
        grid.set_column_spacing(8);
        grid.set_row_spacing(8);
        grid.set_halign(gtk::Align::Center);
        grid.add_css_class("helu-pinpad");

        let buttons = [
            "1", "2", "3",
            "4", "5", "6",
            "7", "8", "9",
            "", "0", "<-",
        ];

        for (i, label) in buttons.iter().enumerate() {
            if label.is_empty() {
                continue;
            }
            let btn = gtk::Button::with_label(label);
            btn.set_size_request(60, 60);
            grid.attach(&btn, (i % 3) as i32, (i / 3) as i32, 1, 1);

            let entry_clone = entry.clone();
            let label = label.to_string();
            btn.connect_clicked(move |_| {
                if label == "<-" {
                    let text = entry_clone.text();
                    if !text.is_empty() {
                        let mut new_text = text.to_string();
                        new_text.pop();
                        entry_clone.set_text(&new_text);
                    }
                } else {
                    let text = entry_clone.text();
                    entry_clone.set_text(&format!("{}{}", text, label));
                }
            });
        }

        let button = gtk::Button::with_label("Sign in");
        button.set_halign(gtk::Align::Center);
        button.add_css_class("suggested-action");

        use std::rc::Rc;
        let submit_pin_action = Rc::new({
            let entry = entry.clone();
            move || {
                let pin = entry.text().to_string();
                // Clear the PIN entry field immediately after emit regardless of result
                entry.set_text("");
                if !pin.is_empty() {
                    // Send PIN to daemon
                    tokio::spawn(async move {
                        // "user" should ideally come from state, but assuming single user logic or daemon knows
                        if let Err(e) = crate::dbus::submit_pin("user", &pin).await {
                            eprintln!("Failed to submit PIN: {:?}", e);
                        }
                    });
                }
            }
        });

        button.connect_clicked({
            let submit_pin_action = submit_pin_action.clone();
            move |_| {
                submit_pin_action();
            }
        });

        entry.connect_activate(move |_| {
            submit_pin_action();
        });

        Self { entry, grid, button }
    }

    pub fn widget(&self) -> gtk::Widget {
        let box_widget = gtk::Box::new(gtk::Orientation::Vertical, 16);
        box_widget.append(&self.entry);
        box_widget.append(&self.grid);
        box_widget.append(&self.button);
        box_widget.upcast()
    }

    pub fn focus(&self) {
        self.entry.grab_focus();
    }
}
