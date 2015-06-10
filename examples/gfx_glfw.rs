extern crate conrod;
extern crate find_folder;
extern crate glfw;
extern crate gfx;
extern crate gfx_window_glfw;
extern crate gfx_graphics;
extern crate viewport;
extern crate piston;

use conrod::{Background, Button, color, Colorable, Labelable, Sizeable, Theme, Ui, Widget, Positionable};
use gfx_graphics::{Gfx2d, GlyphCache};
use gfx::traits::{Stream, Output};

fn main() {

    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
    glfw.set_error_callback(glfw::FAIL_ON_ERRORS);
    glfw.window_hint(glfw::WindowHint::ContextVersion(3, 2));
    glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core));

    let (mut window, events) = glfw.create_window(200, 100, "Hello Conrod", glfw::WindowMode::Windowed).unwrap();

    window.set_key_polling(true);
    window.set_mouse_button_polling(true);

    let (mut stream, mut device, mut factory) = gfx_window_glfw::init(window);

    let assets = find_folder::Search::Both(3, 3).for_folder("assets").unwrap();
    let font_path = assets.join("fonts/NotoSans/NotoSans-Regular.ttf");
    let theme = Theme::default();
    let glyph_cache = GlyphCache::new(&font_path, factory.clone()).unwrap();
    let ui = &mut Ui::new(glyph_cache, theme);

    let mut gfx2d = Gfx2d::new(&mut factory);

    let mut count: u32 = 0;

    while !stream.out.window.should_close() {
        glfw.poll_events();

        for(_, _event) in glfw::flush_messages(&events) {
            //println!("{:?}", event);
            // TODO: Translate glfw events to something that implements event::generic_event::GenericEvent (ie input::Input)
            //       let input_event = input::Input::Press(input::Button::Keyboard(input::Key::Return));
            //       ui.handle_event(&input_event);
        }

        stream.clear(gfx::ClearData {
            color: [0.0, 0.0, 0.0, 1.0],
            depth: 1.0,
            stencil: 0,
        });

        {
            let (renderer, output) = stream.access();

            let (w, h) = output.get_size();
            let viewport = viewport::Viewport {
                rect: [0, 0, w as i32, h as i32],
                draw_size: [w as u32, h as u32],
                window_size: [w as u32, h as u32],
            };

            gfx2d.draw(renderer, output, viewport, |context, graphics| {
                // Draw the background.
                Background::new().rgb(0.2, 0.25, 0.4).draw(ui, graphics);

                // Draw the button and increment count if pressed..
                Button::new()
                    .color(color::red())
                    .dimensions(80.0, 80.0)
                    .bottom_right()
                    .label(&count.to_string())
                    .react(|| count += 1)
                    .set(0, ui);

                // Draw our Ui!
                ui.draw(context, graphics);
            });
        }

        stream.present(&mut device);
    }

}
