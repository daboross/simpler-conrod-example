extern crate conrod;
extern crate glium;
extern crate glutin;
extern crate rusttype;

use conrod::{color, Borderable, Colorable, Positionable};
use conrod::widget::*;

mod glutin_glue;

// --
// In a real app, App would likely be split out into its own module, as would the layout code.
//
// Things are kept together in main.rs here for simplicity.
// --

/// Holds everything we need for application state
struct App {
    /// The conrod UI state
    ui: conrod::Ui,
    /// The handle to the actual window display
    display: glium::Display,
    /// A map of all images conrod can render
    /// Unused unless you render images
    image_map: conrod::image::Map<glium::texture::Texture2d>,
    /// The state of what IDs we know of
    ids: Ids,
    /// The conrod renderer
    renderer: conrod::backend::glium::Renderer,
    // In my applications, I also have a few other things here,
    // representing the app's state itself.
}

impl App {
    pub fn new(window: glium::Display) -> Self {
        let (width, height) = window
            .gl_window()
            .window()
            .get_inner_size()
            .expect("expected getting window size to succeed.");

        // Create UI.
        let mut ui = conrod::UiBuilder::new([width as f64, height as f64]).build();
        let renderer = conrod::backend::glium::Renderer::new(&window)
            .expect("expected loading conrod glium renderer to succeed.");
        let image_map = conrod::image::Map::new();

        let ids = Ids::new(ui.widget_id_generator());

        App {
            ui: ui,
            display: window,
            image_map: image_map,
            ids: ids,
            renderer: renderer,
        }
    }
}

fn load_font() -> rusttype::Font<'static> {
    let font_data = include_bytes!("../OpenSans-Regular.ttf");
    let collection = rusttype::FontCollection::from_bytes(font_data as &[u8]);

    collection
        .into_font()
        .expect("expected loading embedded OpenSans-Regular.ttf font to succeed")
}

fn init_window() -> (glutin::EventsLoop, App) {
    // Create window.
    let events_loop = glutin::EventsLoop::new();
    let window = glutin::WindowBuilder::new()
        .with_dimensions(640, 480)
        .with_title("the-conrod-application");

    let context = glutin::ContextBuilder::new()
        .with_vsync(true)
        .with_multisampling(4);

    let display = glium::Display::new(window, context, &events_loop)
        .expect("expected initial window creation to succeed");

    let mut app = App::new(display);

    // Add font.
    app.ui.fonts.insert(load_font());

    (events_loop, app)
}

fn main() {
    let (glutin_loop, mut app) = init_window();

    let mut events = glutin_glue::EventLoop::new(glutin_loop);

    events.run_loop(|control, glue_event| {
        match glue_event {
            glutin_glue::Event::Glutin(event) => {
                // Pass event onto conrod
                if let Some(conrod_event) =
                    conrod::backend::winit::convert_event(event.clone(), &app.display)
                {
                    app.ui.handle_event(conrod_event);
                    control.needs_update(); // let our event loop know we need to update.
                }

                match event {
                    glutin::Event::WindowEvent { event, .. } => {
                        match event {
                            // When the escape key is pressed or the window is closed, leave the event loop.
                            glutin::WindowEvent::KeyboardInput {
                                input:
                                    glutin::KeyboardInput {
                                        virtual_keycode: Some(glutin::VirtualKeyCode::Escape),
                                        ..
                                    },
                                ..
                            }
                            | glutin::WindowEvent::Closed => control.exit(),
                            // When the window is resized or re-focused, redraw the contents
                            glutin::WindowEvent::Refresh | glutin::WindowEvent::Resized(..) => {
                                app.ui.needs_redraw();
                                control.needs_update();
                            }
                            _ => (),
                        }
                    }
                    // This isn't used now, but if you ever do multi-threading, this will let
                    // other threads awaken the glutin window thread and cause a redraw.
                    glutin::Event::Awakened => {
                        app.ui.needs_redraw();
                        control.needs_update();
                    }
                    _ => (),
                }
            }
            // This event is something our glutin_glue EventLoop creates for any situation
            // where we need to redraw the conrod UI.
            glutin_glue::Event::UpdateUi => {
                create_ui(&mut app);

                if let Some(primitives) = app.ui.draw_if_changed() {
                    let mut target = app.display.draw();

                    app.renderer.fill(&app.display, primitives, &app.image_map);

                    app.renderer
                        .draw(&app.display, &mut target, &app.image_map)
                        .expect("expected drawing GUI to display to succeed");

                    target
                        .finish()
                        .expect("expected frame to remain unfinished before calling finish.")
                }
            }
        }
    })
}

/// Actually do the conrod stuff!
fn create_ui(app: &mut App) {
    let ui = &mut app.ui.set_widgets();
    let ids = &app.ids;

    Canvas::new()
        .color(color::GREY)
        .border(0.0)
        .set(ids.root, ui);

    Text::new("hello")
        .color(color::BLACK)
        .middle_of(ids.root)
        .set(ids.label, ui);
}

// // This creates a structure and a constructor which takes in a ID generator.
// //
// // The structure itself is just a list of IDs. You can do it manually too,
// // though that's more code.
// widget_ids! {
//     struct Ids {
//         root,
//         label,
//     }
// }
// would create this:
struct Ids {
    root: conrod::widget::Id,
    label: conrod::widget::Id,
}

impl Ids {
    pub fn new(mut gen: conrod::widget::id::Generator) -> Self {
        Ids {
            root: gen.next(),
            label: gen.next(),
        }
    }
}
