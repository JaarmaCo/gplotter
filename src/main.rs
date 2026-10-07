use gtk::prelude::*;

fn layout(window: &gtk::ApplicationWindow) {
    let grid = gtk::Grid::new();
    
    let equation_view = gtk::ListBox::new();
    equation_view.set_hexpand(true);
    equation_view.set_vexpand(true);

    for i in 0..10 {
        let cont: String = format!("Button {}", i);
        let button = gtk::Button::with_label(&cont);
        let row = gtk::ListBoxRow::new();
   
        row.set_child(Some(&button));
        equation_view.append(&row);
    }
    
    let draw_space = gtk::Grid::new();

    draw_space.set_hexpand(true);
    draw_space.set_vexpand(true);

    let tool_menu = gtk::Button::with_label("Tool menu");
    tool_menu.set_hexpand(true);
    tool_menu.set_vexpand(false);

    let plot_canvas = gtk::Button::with_label("Canvas");
    plot_canvas.set_vexpand(true);
    plot_canvas.set_hexpand(true);

    draw_space.attach(&tool_menu, 0, 0, 1, 1);
    draw_space.attach(&plot_canvas, 0, 1, 1, 1);

    grid.attach(&equation_view, 0, 0, 1, 1); 
    grid.attach(&draw_space, 1, 0, 3, 1);

    window.set_child(Some(&grid));
}

fn main() {
    let app = gtk::Application::builder()
        .application_id("com.github.jaarmaco.gplotter")
        .build();
    app.connect_activate(|app| {
        let window = gtk::ApplicationWindow::new(app);
        window.set_default_size(1024, 512);

        layout(&window);

        window.present();
    });
    app.run();
}
