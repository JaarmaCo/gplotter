use std::fmt::Display;
use std::rc::Rc;
use std::cell::RefCell;
use std::vec::Vec;

use gtk::prelude::*;
use gtk::gdk;
use libm::*;

macro_rules! cfor {
    ($init:stmt; $cond:expr; $step:expr; $body:block) => {{
        $init
        while $cond {
            $body
            $step
        }
    }};
}

#[derive(Debug, Clone)]
enum Equation {
    Dep,
    Add { lhs: Rc<Equation>, rhs: Rc<Equation> },
    Sub { lhs: Rc<Equation>, rhs: Rc<Equation> },
    Mul { lhs: Rc<Equation>, rhs: Rc<Equation> },
    Div { lhs: Rc<Equation>, rhs: Rc<Equation> },
    Exp { lhs: Rc<Equation>, rhs: Rc<Equation> },
    Func { name: String, args: Vec<Rc<Equation>> },
    Const { value: f64 },
}

impl Display for Equation {
    fn fmt(&self, ctx: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Equation::Dep => ctx.write_str("x"),
            Equation::Add { lhs, rhs } => write!(ctx, "(+ {} {})", *lhs, *rhs),
            Equation::Sub { lhs, rhs } => write!(ctx, "(- {} {})", *lhs, *rhs),
            Equation::Mul { lhs, rhs } => write!(ctx, "(* {} {})", *lhs, *rhs),
            Equation::Div { lhs, rhs } => write!(ctx, "(/ {} {})", *lhs, *rhs),
            Equation::Exp { lhs, rhs } => write!(ctx, "(^ {} {})", *lhs, *rhs),
            Equation::Func { name, args } => {
                write!(ctx, "({} ", name)?; 
                if args.len() == 0 {
                    ctx.write_str("[])")
                } else if args.len() == 0 {
                    write!(ctx, "[ {} ])", *args[0])
                } else {
                    write!(ctx, "[ {}", *args[0])?;
                    for arg in &args.as_slice()[1..] {
                        write!(ctx, ", {}", *arg)?;
                    }
                    ctx.write_str(" ])")
                }
            },
            Equation::Const { value } => write!(ctx, "{}", value),
        }
    }
}

#[derive(Debug, Clone)]
enum Token {
    X,
    Plus,
    Minus,
    Star,
    Slash,
    Hat,
    OParen,
    CParen,
    Comma,
    FName(String),
    Literal(f64),
}

fn lookup_fn(name: &String) -> Option<Box<dyn Fn(&Vec<f64>) -> f64>> {
    return match name.as_str() {
        "pi" => Some(Box::new(|_| 3.141592653589793238462643383279)),
        "sin" => Some(Box::new(|xs| if xs.len() >= 1 { f64::sin(xs[0]) } else { f64::NAN })),
        "cos" => Some(Box::new(|xs| if xs.len() >= 1 { f64::cos(xs[0]) } else { f64::NAN })),
        "tan" => Some(Box::new(|xs| if xs.len() >= 1 { f64::tan(xs[0]) } else { f64::NAN })),
        "sqrt" => Some(Box::new(|xs| if xs.len() >= 1 { f64::sqrt(xs[0]) } else { f64::NAN })),
        "ln" => Some(Box::new(|xs| if xs.len() >= 1 { f64::ln(xs[0]) } else { f64::NAN })),
        "log" => Some(Box::new(|xs| if xs.len() >= 2 { f64::log(xs[0], xs[1]) } else { f64::NAN })),
        "log10" => Some(Box::new(|xs| if xs.len() >= 1 { f64::log10(xs[0]) } else { f64::NAN })),
        "log2" => Some(Box::new(|xs| if xs.len() >= 1 { f64::log2(xs[0]) } else { f64::NAN })),
        _ => None
    }
}

fn eval(x: f64, eq: &Equation) -> Result<f64, String> {
    match eq {
        Equation::Dep => Ok(x),
        Equation::Add{ lhs, rhs } => Ok(eval(x, &lhs)? + eval(x, &rhs)?),
        Equation::Sub{ lhs, rhs } => Ok(eval(x, &lhs)? - eval(x, &rhs)?),
        Equation::Mul{ lhs, rhs } => Ok(eval(x, &lhs)? * eval(x, &rhs)?),
        Equation::Div{ lhs, rhs } => Ok(eval(x, &lhs)? / eval(x, &rhs)?),
        Equation::Exp{ lhs, rhs } => Ok(pow(eval(x, &lhs)?, eval(x, &rhs)?)),
        Equation::Func{ name, args } => {
            let mut arg_vec = Vec::new();
            if let Some(func) = lookup_fn(&name) {
                for e in args {
                    arg_vec.push(eval(x, &e)?);
                }
                Ok(func(&arg_vec))
            } else {
                Err(format!("Reference to unknown function {}", name))
            }
        },
        Equation::Const { value } => Ok(*value),
    }
}

fn lex_number<'a>(s: &'a str) -> Option<(&'a str, &'a str)> {
    let mut i = 0;
    let mut it = s.chars().peekable();

    while let Some(ch) = it.peek() && ch.is_ascii_digit() {
        i += 1;
        it.next();
    }

    if let Some(ch) = it.peek() && *ch == '.' {
        i += 1;
        it.next();
        while let Some(ch) = it.peek() && ch.is_ascii_digit() {
            i += 1;
            it.next();
        }

        if let Some(ch) = it.peek() && (*ch == 'e' || *ch == 'E') {
            i += 1;
            it.next();
            if let Some(ch) = it.peek() && (*ch == '+' || *ch == '-' || ch.is_ascii_digit()) {
                i += 1;
                it.next();
                while let Some(ch) = it.peek() && ch.is_ascii_digit() {
                    i += 1;
                    it.next();
                }
            }
        }
    }

    if i == 0 {
        None
    } else {
        Some((&s[0..i], &s[i..]))
    }
}

fn lex_ident<'a>(s: &'a str) -> Option<(&'a str, &'a str)> {

    let mut i = 0;
    let mut it = s.chars();
    if let Some(c) = it.next() && c.is_alphabetic() {
        i += 1;
        while let Some(c) = it.next() && (c.is_alphanumeric() || c == '_') {
            i += 1;
        }
    }
    
    if i == 0 {
        None
    } else {
        Some((&s[0..i], &s[i..]))
    }
}

fn lex_eq(chars_in: &str) -> Result<Vec<Token>, String> { 

    let mut res = Vec::new();
    let mut chars = chars_in;
    while !chars.is_empty() {

        chars = chars.trim_start();

        if let Some(rem) = chars.strip_prefix("+") {
            res.push(Token::Plus);
            chars = rem;
        } else if let Some(rem) = chars.strip_prefix("-") {
            res.push(Token::Minus);
            chars = rem;
        } else if let Some(rem) = chars.strip_prefix("*") {
            res.push(Token::Star);
            chars = rem;
        } else if let Some(rem) = chars.strip_prefix("/") {
            res.push(Token::Slash);
            chars = rem;
        } else if let Some(rem) = chars.strip_prefix("^") {
            res.push(Token::Hat);
            chars = rem;
        } else if let Some(rem) = chars.strip_prefix("(") {
            res.push(Token::OParen);
            chars = rem;
        } else if let Some(rem) = chars.strip_prefix(")") {
            res.push(Token::CParen);
            chars = rem;
        } else if let Some(rem) = chars.strip_prefix("x") {
            res.push(Token::X);
            chars = rem;
        } else if let Some(rem) = chars.strip_prefix(",") {
            res.push(Token::Comma);
            chars = rem;
        } else if let Some((fname, rem)) = lex_ident(chars) {
            res.push(Token::FName(String::from(fname)));
            chars = rem;
        } else if let Some((cvalue, rem)) = lex_number(chars) {
            if let Ok(value) = cvalue.parse::<f64>() {
                res.push(Token::Literal(value));
                chars = rem;
            } else {
                return Err(String::from("Invalid number literal"));
            }
        } else {
            return Err(String::from("Token not recognized"));
        }
    }
    Ok(res)
}

fn parse_eq_prim(tokens: &[Token]) -> Result<(&[Token], Equation), String> {
    if tokens.is_empty() {
        Err(String::from("Unexpected end of input"))
    } else {
        match tokens[0].clone() {
            Token::X =>
                Ok((&tokens[1..], Equation::Dep)),
            Token::Literal(value) =>
                Ok((&tokens[1..], Equation::Const{ value: value })),
            Token::OParen => {
                let (rest, eq) = parse_eq(&tokens[1..])?;
                if !rest.is_empty() && let Token::CParen = rest[0] {
                    Ok((&rest[1..], eq))
                } else {
                    Err(String::from("Parenthesis not closed"))
                }
            },
            Token::FName(fname) => {
                if tokens.len() >= 3 && let Token::OParen = tokens[1] {
                    let mut rest = &tokens[2..];
                    let mut args = Vec::new();
                    loop {
                        if rest.is_empty() {
                            break;
                        } else if let Token::CParen = rest[0] {
                            rest = &rest[1..]; 
                            break;
                        }

                        let (next_rest, parsed_arg) = parse_eq(rest)?;
                        if next_rest.is_empty() {
                            return Err(String::from("End of input before the function argument list was closed"));
                        }

                        if let Token::Comma = next_rest[0] {
                            args.push(Rc::new(parsed_arg));
                            rest = &next_rest[1..];
                        } else if let Token::CParen = next_rest[0] {
                            args.push(Rc::new(parsed_arg));
                            rest = &next_rest[1..];
                            break;
                        } else {
                            return Err(String::from("Expected a comma ',' or a closing parenthesis ')'"));
                        }
                    }
                    Ok((rest, Equation::Func{ name: fname, args: args }))
                } else {
                    Err(String::from("Invalid function application, expected: <fname> '(' [ <eq> [ ',' <eq> ]... ] ')'"))
                }
            }
            _ => Err(String::from("Syntax error"))
        }
    }
}

fn parse_eq_2(tokens: &[Token]) -> Result<(&[Token], Equation), String> {
    let (rest, lhs) = parse_eq_prim(tokens)?;
    if rest.is_empty() {
        Ok((rest, lhs))
    } else {
        match rest[0] {
            Token::Hat => {
                let (rest, rhs) = parse_eq_2(&tokens[1..])?;
                Ok((rest, Equation::Exp{ lhs: Rc::new(lhs), rhs: Rc::new(rhs) }))
            },
            _ => Ok((rest, lhs))
        }
    }
}

fn parse_eq_1(tokens: &[Token]) -> Result<(&[Token], Equation), String> {
    let (rest, lhs) = parse_eq_2(tokens)?;
    if rest.is_empty() {
        Ok((rest, lhs))
    } else {
        match rest[0] {
            Token::Star => {
                let (rest, rhs) = parse_eq_1(&rest[1..])?;
                Ok((rest, Equation::Mul{ lhs: Rc::new(lhs), rhs: Rc::new(rhs) }))
            },
            Token::Slash => {
                let (rest, rhs) = parse_eq_1(&rest[1..])?;
                Ok((rest, Equation::Div{ lhs: Rc::new(lhs), rhs: Rc::new(rhs) }))
            },
            _ => Ok((rest, lhs))
        }
    }
}

fn parse_eq(tokens: &[Token]) -> Result<(&[Token], Equation), String> {
    let (rest, lhs) = parse_eq_1(tokens)?;
    if rest.is_empty() {
        Ok((rest, lhs))
    } else {
        match rest[0] {
            Token::Plus => {
                let (rest, rhs) = parse_eq(&rest[1..])?;
                Ok((rest, Equation::Add{ lhs: Rc::new(lhs), rhs: Rc::new(rhs) }))
            }
            Token::Minus => {
                let (rest, rhs) = parse_eq(&rest[1..])?;
                Ok((rest, Equation::Sub{ lhs: Rc::new(lhs), rhs: Rc::new(rhs) }))
            }
            _ =>
                Ok((rest, lhs))
        }
    }
}

fn read_eq(src: &str) -> Result<Equation, String> {
    let tokens = lex_eq(src)?;
    let (_, eq) = parse_eq(&tokens)?;
    Ok(eq)
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

fn draw_background(cairo: &gtk::cairo::Context) {
    cairo.set_source_rgb(1.0, 1.0, 1.0);
    cairo.paint().unwrap();
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

fn draw_scalars(cairo: &gtk::cairo::Context, dx: f64, dy: f64, w: f64, h: f64, scale_x: f64, scale_y: f64) {

    let pad_x = 5.0;
    let pad_y = 5.0;

    cairo.set_font_size(10.0);
    cairo.set_source_rgb(0.0, 0.0, 0.0);

    cairo.move_to(w / 2.0 + pad_x, h / 2.0 - pad_y);
    cairo.show_text("0").unwrap();

    let mut x = w / 2.0 + dx;
    let mut value_x = scale_x;
    while x <= w {

        let label = format!("{:.2}", value_x);

        cairo.move_to(x + pad_x, h / 2.0 - pad_y);
        cairo.show_text(&label).unwrap();

        value_x += scale_x;
        x += dx;
    }

    x = w / 2.0 - dx;
    value_x = -scale_x;
    while x >= 0.0 {

        let label = format!("{:.2}", value_x);

        cairo.move_to(x + pad_x, h / 2.0 - pad_y);
        cairo.show_text(&label).unwrap();

        value_x -= scale_x;
        x -= dx;
    }

    let mut y = h / 2.0 - dy;
    let mut value_y = scale_y;
    while y >= 0.0 {

        let label = format!("{:.2}", value_y);

        cairo.move_to(w / 2.0 + pad_x, y - pad_y);
        cairo.show_text(&label).unwrap();

        value_y += scale_y;
        y -= dy;
    }

    y = h / 2.0 + dy;
    value_y = -scale_y;
    while y <= h {

        let label = format!("{:.2}", value_y);

        cairo.move_to(w / 2.0 + pad_x, y - pad_y);
        cairo.show_text(&label).unwrap();

        value_y -= scale_y;
        y += dy;
    }
}

fn draw_equation<F: Fn(f64) -> f64>(cairo: &gtk::cairo::Context, w: f64, h: f64, equation: &F) {
    cairo.set_source_rgb(1.0, 0.0, 0.0);
    cairo.set_line_width(5.0);

    let mut x = -1.0;
    cairo.move_to(w / 2.0 + x * h, h / 2.0 - equation(x) * h);
    for _ in 0 .. 2000 {
        let y = equation(x);

        let abs_x = w / 2.0 + x * h;
        let abs_y = h / 2.0 - y * h;

        cairo.line_to(abs_x, abs_y);

        x += 0.001;
    }
    cairo.stroke().unwrap();
}

fn equation_box<T: IsA<gtk::Widget>>(eq_list: Rc<RefCell<Vec<Equation>>>, hook: &T) -> gtk::Text {
    let buffer = gtk::EntryBuffer::new(Some(""));
    let view = gtk::Text::with_buffer(&buffer);

    view.set_editable(true);
    view.set_hexpand(true);
    view.set_placeholder_text(Some("Enter equation..."));
    view.add_css_class("ebox");

    eq_list.borrow_mut().push(Equation::Const { value: 0.0 });
    
    let index = eq_list.borrow().len() - 1;
    let local_hook = hook.clone();
    view.connect_changed(move |view|
        {
            println!("Text changed!");
            match read_eq(&view.text()) {
                Ok(eq) => {
                    println!("Parsed eq: {}", eq);
                    eq_list.borrow_mut()[index] = eq;
                    local_hook.queue_draw();
                },
                Err(msg) =>
                    println!("{}", msg)
            }
        });    
    view
}

fn layout(window: &gtk::ApplicationWindow) {
    let grid = gtk::Grid::new();
    
    let equations = Rc::new(RefCell::new(Vec::new()));

    let equation_view = gtk::ListBox::new();
    equation_view.set_hexpand(true);
    equation_view.set_vexpand(true);

    let plot_canvas = gtk::DrawingArea::new();
    plot_canvas.set_vexpand(true);
    plot_canvas.set_hexpand(true);

    let item = equation_box(equations.clone(), &plot_canvas);
    let row = gtk::ListBoxRow::new();

    row.set_child(Some(&item));
    equation_view.append(&row);

    let draw_space = gtk::Grid::new();

    draw_space.set_hexpand(true);
    draw_space.set_vexpand(true);

    let tool_menu = gtk::Button::with_label("Tool menu");
    tool_menu.set_hexpand(true);
    tool_menu.set_vexpand(false);

    let eq_list = equations.clone();
    plot_canvas.set_draw_func(move |_, cairo, width, height|{

        let w  = width as f64;
        let h  = height as f64;

        draw_background(&cairo);
        draw_grid(&cairo, w, h, w / 20.0, w / 20.0);
        draw_axes(&cairo, w, h);
        draw_scalars(&cairo, w / 20.0, w / 20.0, w, h, 1.0 / 20.0, 1.0 / 20.0);
        
        let eq_ref = eq_list.borrow();
        let iter = eq_ref.iter();
        for eq in iter {
            let eq_cp = eq.clone();
            if let Err(msg) = eval(0.0, &eq_cp) {
                println!("Eval error: {}", msg);
                break;
            }
            draw_equation(&cairo, w, h,
                &move |x| {
                    match eval(x, &eq_cp) {
                        Ok(y) => y,
                        Err(msg) => {
                            println!("Eval error: {}", msg);
                            0.0
                        }
                    }
                });
        }
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
 
        let css_provider = gtk::CssProvider::new();
        css_provider.load_from_data(include_str!("../styles/main.css"));
        let prio = gtk::STYLE_PROVIDER_PRIORITY_USER;
        let display = gdk::Display::default().expect("Failed to find the default display.");

        gtk::style_context_add_provider_for_display(&display, &css_provider, prio);

        window.present();
    });
    app.run();
}
