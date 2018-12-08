#![windows_subsystem = "windows"]
#![warn(bare_trait_objects)]

use ggez::{
    conf, event,
    filesystem::Filesystem,
    graphics::{self, Point2, Rect},
    Context, ContextBuilder, GameResult,
};
use log::info;
use structopt::StructOpt;

mod core;
mod geom;
mod screen;
mod utils;

// TODO: https://github.com/ggez/ggez/issues/384
type ZResult<T = ()> = GameResult<T>;

const APP_ID: &str = "zemeroth";
const APP_AUTHOR: &str = "ozkriff";
const ASSETS_DIR_NAME: &str = "assets";
const ASSETS_HASHSUM: &str = "1756a424ff728bfa10679d381ab02a17";

struct MainState {
    screens: screen::Screens,
}

impl MainState {
    fn new(context: &mut Context) -> ZResult<Self> {
        let start_screen = Box::new(screen::MainMenu::new(context)?);
        let screens = screen::Screens::new(start_screen);
        let mut this = Self { screens };
        {
            let (w, h) = graphics::get_drawable_size(context);
            this.resize(context, w, h);
        }
        Ok(this)
    }

    fn resize(&mut self, context: &mut Context, w: u32, h: u32) {
        let aspect_ratio = w as f32 / h as f32;
        let coordinates = Rect::new(-aspect_ratio, -1.0, aspect_ratio * 2.0, 2.0);
        graphics::set_screen_coordinates(context, coordinates).unwrap();
        self.screens.resize(aspect_ratio);
    }
}

impl event::EventHandler for MainState {
    fn update(&mut self, context: &mut Context) -> ZResult {
        self.screens.update(context)
    }

    fn draw(&mut self, context: &mut Context) -> ZResult {
        self.screens.draw(context)
    }

    fn resize_event(&mut self, context: &mut Context, w: u32, h: u32) {
        self.resize(context, w, h);
    }

    fn mouse_button_up_event(
        &mut self,
        context: &mut Context,
        _: ggez::event::MouseButton,
        x: i32,
        y: i32,
    ) {
        let window_pos = Point2::new(x as _, y as _);
        let pos = ui::window_to_screen(context, window_pos);
        self.screens
            .click(context, pos)
            .expect("Can't handle click event");
    }

    // This functions just overrides the default implementation,
    // because we don't want to quit from the game on `Esc`.
    fn key_down_event(&mut self, _: &mut Context, _: event::Keycode, _: event::Mod, _: bool) {}
}

fn context() -> Context {
    let window_conf = conf::WindowSetup::default()
        .resizable(true)
        .title("Zemeroth");
    ContextBuilder::new(APP_ID, APP_AUTHOR)
        .window_setup(window_conf)
        .add_resource_path(ASSETS_DIR_NAME)
        .build()
        .expect("Can't build context")
}

fn fs() -> Filesystem {
    let mut fs = Filesystem::new(APP_ID, APP_AUTHOR).expect("Can't create a filesystem");
    fs.mount(std::path::Path::new(ASSETS_DIR_NAME), true);
    fs
}

#[derive(StructOpt, Debug)]
#[structopt(name = "Zemeroth")]
struct Options {
    /// Only check assets' hash
    #[structopt(long = "check-assets")]
    check_assets: bool,
}

fn main() -> ZResult {
    let opt = Options::from_args();
    env_logger::init();
    enable_backtrace();
    info!("Checking assets hash file...");
    utils::check_assets_hash(&mut fs(), ASSETS_HASHSUM)?;
    if opt.check_assets {
        // That's it. We don't need to run the game itself
        return Ok(());
    }
    info!("Creating context...");
    let mut context = context();
    info!("Creating MainState...");
    let mut state = MainState::new(&mut context)?;
    info!("Starting the main loop...");
    event::run(&mut context, &mut state)
}

fn enable_backtrace() {
    if std::env::var("RUST_BACKTRACE").is_err() {
        std::env::set_var("RUST_BACKTRACE", "1");
    }
}
