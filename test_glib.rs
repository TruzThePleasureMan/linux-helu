use gtk::glib;
fn main() {
    let (tx, rx) = glib::MainContext::channel::<()>(glib::Priority::DEFAULT);
}
