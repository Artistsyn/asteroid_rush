// quartz_forge-managed: build_app scaffold
use quartz::*;
use ramp::prism;
use ramp::Drawable;

#[path = "scenes/main_scene.rs"]
mod generated_scene;

pub struct App;

impl App {
    fn new(ctx: &mut Context) -> impl Drawable {
        let mut canvas = Canvas::new(ctx, CanvasMode::Landscape);
        generated_scene::setup_scene(&mut canvas);
        generated_scene::register_logic(&mut canvas);
        generated_scene::register_events(&mut canvas);
        canvas
    }
}

ramp::run! { []; |ctx: &mut Context| { App::new(ctx) } }
