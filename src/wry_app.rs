use std::{
    error::Error,
    io,
    net::{SocketAddr, TcpStream},
    process::{Child, Command, Stdio},
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use tao::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use wry::WebViewBuilder;

const HOST: &str = "127.0.0.1";
const PORT: u16 = 8080;
const URL: &str = "http://127.0.0.1:8080";

pub fn run() -> Result<(), Box<dyn Error>> {
    let server = Arc::new(Mutex::new(Some(spawn_trunk_serve()?)));
    wait_for_server()?;

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("gnostr-wasm")
        .build(&event_loop)?;

    let _webview = WebViewBuilder::new().with_url(URL).build(&window)?;

    let server = Arc::clone(&server);
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        if let Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } = event
        {
            if let Some(mut child) = server.lock().ok().and_then(|mut slot| slot.take()) {
                let _ = child.kill();
                let _ = child.wait();
            }
            *control_flow = ControlFlow::Exit;
        }
    });
}

fn spawn_trunk_serve() -> Result<Child, io::Error> {
    Command::new("trunk")
        .args([
            "serve",
            "--address",
            HOST,
            "--port",
            "8080",
            "--no-autoreload",
            "true",
            "--open",
            "false",
        ])
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
}

fn wait_for_server() -> Result<(), io::Error> {
    let addr: SocketAddr = format!("{HOST}:{PORT}").parse().map_err(|err| {
        io::Error::new(io::ErrorKind::InvalidInput, format!("invalid address: {err}"))
    })?;

    for _ in 0..120 {
        if TcpStream::connect_timeout(&addr, Duration::from_millis(250)).is_ok() {
            return Ok(());
        }
        thread::sleep(Duration::from_millis(500));
    }

    Err(io::Error::new(
        io::ErrorKind::TimedOut,
        "trunk serve did not start on 127.0.0.1:8080",
    ))
}
