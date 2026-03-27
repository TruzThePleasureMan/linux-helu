use gtk4::glib;
use gtk4::glib::clone;
fn main() {
    let (tx, rx) = async_channel::unbounded::<()>();

    let ctx = glib::MainContext::default();
    ctx.spawn_local(clone!(
        #[strong] rx,
        async move {
            while let Ok(msg) = rx.recv().await {
                println!("Got msg");
            }
        }
    ));
}
