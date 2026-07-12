//! # [Ratatui] Original Demo example
//!
//! The latest version of this example is available in the [examples] folder in the upstream.
//!
//! [Ratatui]: https://github.com/ratatui/ratatui
//! [examples]: https://github.com/ratatui/ratatui/blob/main/examples
//! [examples readme]: https://github.com/ratatui/ratatui/blob/main/examples/README.md

#[cfg(feature = "wry-app")]
mod wry_app;

#[cfg(not(feature = "wry-app"))]
use std::{cell::RefCell, io::Result, rc::Rc};

#[cfg(not(feature = "wry-app"))]
use app::App;
#[cfg(not(feature = "wry-app"))]
use backend::{BackendType, MultiBackendBuilder};
#[cfg(not(feature = "wry-app"))]
use ratzilla::backend::webgl2::WebGl2BackendOptions;
#[cfg(not(feature = "wry-app"))]
use ratzilla::event::KeyCode;
#[cfg(not(feature = "wry-app"))]
use ratzilla::WebRenderer;

#[cfg(not(feature = "wry-app"))]
mod app;
#[cfg(not(feature = "wry-app"))]
mod backend;
#[cfg(not(feature = "wry-app"))]
mod fps;
#[cfg(not(feature = "wry-app"))]
mod effects;
#[cfg(not(feature = "wry-app"))]
mod ui;
#[cfg(not(feature = "wry-app"))]
mod utils;

#[cfg(not(feature = "wry-app"))]
fn main() -> Result<()> {
    let app_state = Rc::new(RefCell::new(App::new("Demo", true)));

    let webgl2_options = WebGl2BackendOptions::new()
        .measure_performance(true)
        .enable_console_debug_api()
        .enable_mouse_selection()
        .disable_auto_css_resize(); // canvas size managed by css in index.html

    let mut terminal = MultiBackendBuilder::with_fallback(BackendType::WebGl2)
        .webgl2_options(webgl2_options)
        .build_terminal()?;

    terminal.on_key_event({
        let app_state_cloned = app_state.clone();
        move |event| {
            let mut app_state = app_state_cloned.borrow_mut();
            match event.code {
                KeyCode::Right => {
                    app_state.on_right();
                }
                KeyCode::Left => {
                    app_state.on_left();
                }
                KeyCode::Up => {
                    app_state.on_up();
                }
                KeyCode::Down => {
                    app_state.on_down();
                }
                KeyCode::Char(c) => app_state.on_key(c),
                _ => {}
            }
        }
    })?;

    terminal.draw_web(move |f| {
        let mut app_state = app_state.borrow_mut();
        let elapsed = app_state.on_tick();
        ui::draw(elapsed, f, &mut app_state);
    });

    Ok(())
}

#[cfg(feature = "wry-app")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    wry_app::run()
}
