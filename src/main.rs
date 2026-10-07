use gtk::glib;
use gtk::prelude::*;

fn main() {
    let app = gtk::Application::builder()
        .application_id("com.github.jaarmaco.gplotter")
        .build();
    app.connect_activate(|app| {
        let window = gtk::ApplicationWindow::new(app);
        window.set_default_size(1024, 512);

        let button = gtk::Button::with_label("Hello, world!");

        button.connect_clicked(glib::clone!(#[weak] window, move |_| window.close()));

        window.set_child(Some(&button));
        window.present();
    });
    app.run();
}
