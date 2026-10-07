use gtk::prelude::*;

macro_rules! cfor {
    ($init:stmt; $cond:expr; $step:expr; $body:block) => {{
        $init
        while $cond {
            $body
            $step
        }
    }};
}

fn equation(x: f64) -> f64 {
    x * x
}

fn draw_grid(cairo: &gtk::cairo::Context, w: f64, h: f64, dx: f64, dy: f64) {

    cairo.set_source_rgb(0.0, 0.0, 0.0);
    cairo.set_line_width(1.0);

    cfor!(let mut x = w / 2.0 - dx; x >= 0.0; x -= dx; {
        cairo.move_to(x, 0.0);
        cairo.line_to(x, h);
    });

    cfor!(let mut x = w / 2.0 + dx; x <= w; x += dx; {
        cairo.move_to(x, 0.0);
        cairo.line_to(x, h);
    });

    cfor!(let mut y = h / 2.0 - dy; y >= 0.0; y -= dy; {
        cairo.move_to(0.0, y);
        cairo.line_to(w, y);
    });

    cfor!(let mut y = h / 2.0 + dy; y <= h; y += dy; {
        cairo.move_to(0.0, y);
        cairo.line_to(w, y);
    });

    cairo.stroke().unwrap();
}

fn draw_axes(cairo: &gtk::cairo::Context, w: f64, h: f64) {
    cairo.set_source_rgb(0.0, 0.0, 0.0);
    cairo.set_line_width(3.0);

    cairo.move_to(w / 2.0, 0.0);
    cairo.line_to(w / 2.0, h);

    cairo.move_to(0.0, h / 2.0);
    cairo.line_to(w, h / 2.0);

    cairo.stroke().unwrap();
}

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

    let plot_canvas = gtk::DrawingArea::new();
    plot_canvas.set_vexpand(true);
    plot_canvas.set_hexpand(true);
    plot_canvas.set_draw_func(|_, cairo, width, height|{
    
        let w  = width as f64;
        let h  = height as f64;

        cairo.set_source_rgb(1.0, 1.0, 1.0);
        cairo.paint().unwrap();

        draw_grid(&cairo, w, h, w / 20.0, w / 20.0);
        draw_axes(&cairo, w, h);

        cairo.set_source_rgb(1.0, 0.0, 0.0);
        cairo.set_line_width(5.0);

        let mut x = -1.0;
        for _ in 0 .. 2000 {
            let y = equation(x);

            let abs_x = w / 2.0 + x * h;
            let abs_y = h / 2.0 - y * h;

            cairo.line_to(abs_x, abs_y);

            x += 0.001;
        }
        cairo.stroke().unwrap();
    });

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
