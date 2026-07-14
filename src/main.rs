#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

//! # [Ratatui] Original Demo example
//!
//! The latest version of this example is available in the [examples] folder in the upstream.
//!
//! [Ratatui]: https://github.com/ratatui/ratatui
//! [examples]: https://github.com/ratatui/ratatui/blob/main/examples
//! [examples readme]: https://github.com/ratatui/ratatui/blob/main/examples/README.md

#[cfg(feature = "wry-app")]
mod wry_app;

use std::error::Error;

#[cfg(all(not(feature = "wry-app"), target_arch = "wasm32"))]
use std::{cell::RefCell, rc::Rc};

#[cfg(all(not(feature = "wry-app"), target_arch = "wasm32"))]
use gnostr_wasm::app::App;
#[cfg(all(not(feature = "wry-app"), target_arch = "wasm32"))]
use gnostr_wasm::backend::{BackendType, MultiBackendBuilder};
#[cfg(all(not(feature = "wry-app"), target_arch = "wasm32"))]
use ratzilla::backend::webgl2::WebGl2BackendOptions;
#[cfg(all(not(feature = "wry-app"), target_arch = "wasm32"))]
use ratzilla::event::KeyCode;
#[cfg(all(not(feature = "wry-app"), target_arch = "wasm32"))]
use ratzilla::WebRenderer;

#[cfg(all(not(feature = "wry-app"), target_arch = "wasm32"))]
fn main() -> Result<(), Box<dyn Error>> {
    browser_main()
}

#[cfg(all(not(feature = "wry-app"), target_arch = "wasm32"))]
fn browser_main() -> Result<(), Box<dyn Error>> {
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
        gnostr_wasm::ui::draw(elapsed, f, &mut app_state);
    });

    Ok(())
}

#[cfg(feature = "wry-app")]
fn main() -> Result<(), Box<dyn Error>> {
    wry_app::run()
}

#[cfg(all(not(feature = "wry-app"), not(target_arch = "wasm32")))]
fn main() {
    println!(
        "gnostr-wasm's browser entrypoint is for wasm32.\n\
         Run `trunk serve` or `trunk build` for the Ratzilla web app."
    );
}
