use gtk4 as gtk;
use gtk::prelude::*;
use std::rc::Rc;
use std::cell::RefCell;
use std::f64::consts::PI;
use gtk::glib;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RingState {
    Idle,
    Scanning,
    Success,
    Failure,
    FidoPrompt,
    PinFallback,
}

pub struct PulsingRing {
    area: gtk::DrawingArea,
    state: RingState,
    pulse: f64,
}

impl Default for PulsingRing {
    fn default() -> Self {
        Self::new()
    }
}

impl PulsingRing {
    pub fn new() -> Self {
        let area = gtk::DrawingArea::new();
        area.set_size_request(200, 200);

        let this = Rc::new(RefCell::new(Self {
            area: area.clone(),
            state: RingState::Idle,
            pulse: 0.0,
        }));

        area.set_draw_func({
            let this = this.clone();
            move |_, cr, width, height| {
                this.borrow().draw(cr, width as f64, height as f64);
            }
        });

        // Set up animation timer
        glib::source::timeout_add_local(std::time::Duration::from_millis(16), {
            let this = this.clone();
            move || {
                let mut ring = this.borrow_mut();
                if ring.state == RingState::Scanning {
                    ring.pulse = (ring.pulse + 0.05) % (PI * 2.0);
                    ring.area.queue_draw();
                } else if ring.state == RingState::FidoPrompt {
                    ring.pulse = (ring.pulse + 0.02) % (PI * 2.0);
                    ring.area.queue_draw();
                }
                glib::ControlFlow::Continue
            }
        });

        let result = this.borrow().clone();
        result
    }

    pub fn widget(&self) -> &gtk::Widget {
        self.area.upcast_ref()
    }

    pub fn set_state(&mut self, state: RingState) {
        self.state = state;
        self.area.queue_draw();
    }

    fn draw(&self, cr: &cairo::Context, width: f64, height: f64) {
        let x = width / 2.0;
        let y = height / 2.0;
        let radius = 80.0;

        let (r, g, b) = match self.state {
            RingState::Idle => (0.267, 0.267, 0.267), // #444444
            RingState::Scanning => {
                let intensity = (self.pulse.sin() + 1.0) / 2.0;
                (0.2, 0.6 + 0.4 * intensity, 1.0) // pulsing blue
            }
            RingState::Success => (0.133, 0.91, 0.478), // #22e87a
            RingState::Failure => (0.91, 0.267, 0.133), // #e84422
            RingState::FidoPrompt => {
                let intensity = (self.pulse.sin() + 1.0) / 2.0;
                (0.91, 0.753 + 0.1 * intensity, 0.133) // #e8c022 with slow pulse
            }
            RingState::PinFallback => (0.267, 0.267, 0.267), // #444444
        };

        cr.set_source_rgba(r, g, b, 0.8);
        cr.set_line_width(8.0);
        cr.arc(x, y, radius, 0.0, PI * 2.0);
        cr.stroke().unwrap();
    }
}

// Implement Clone so we can return `this.borrow().clone()` safely.
impl Clone for PulsingRing {
    fn clone(&self) -> Self {
        Self {
            area: self.area.clone(),
            state: self.state,
            pulse: self.pulse,
        }
    }
}
