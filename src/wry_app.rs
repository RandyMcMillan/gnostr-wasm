use std::{error::Error, io, path::PathBuf};

use tao::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use url::Url;
use wry::WebViewBuilder;

pub fn run() -> Result<(), Box<dyn Error>> {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("gnostr-wasm")
        .build(&event_loop)?;

    let index_path = std::env::current_dir()?.join(PathBuf::from("dist/index.html"));
    let index_url = Url::from_file_path(&index_path).map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("invalid file path: {}", index_path.display()),
        )
    })?;

    let _webview = WebViewBuilder::new()
        .with_url(index_url.as_str())
        .build(&window)?;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        if let Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } = event
        {
            *control_flow = ControlFlow::Exit;
        }
    });
}
