#[cfg(feature = "wry-app")]
use std::{
    error::Error,
    io,
    net::{SocketAddr, TcpStream},
    process::{Child, Command, Stdio},
    thread,
    time::Duration,
};

#[cfg(feature = "wry-app")]
use tao::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
#[cfg(feature = "wry-app")]
use wry::WebViewBuilder;

#[cfg(feature = "wry-app")]
const HOST: &str = "127.0.0.1";
#[cfg(feature = "wry-app")]
const PORT: u16 = 8080;
#[cfg(feature = "wry-app")]
const URL: &str = "http://127.0.0.1:8080";

#[cfg(feature = "wry-app")]
fn main() -> Result<(), Box<dyn Error>> {
    let mut trunk = spawn_trunk_serve()?;
    wait_for_server()?;

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("gnostr-cloud")
        .build(&event_loop)?;

    let _webview = WebViewBuilder::new().with_url(URL).build(&window)?;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        if let Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } = event
        {
            let _ = trunk.kill();
            let _ = trunk.wait();
            *control_flow = ControlFlow::Exit;
        }
    })
}

#[cfg(not(feature = "wry-app"))]
fn main() {
    println!(
        "gnostr-cloud is a desktop wrapper for the Ratzilla web app.\n\
         Run `cargo run --bin gnostr-cloud --features wry-app` to launch it,\n\
         or run `trunk serve` / `trunk build` to use the web UI directly."
    );
}

#[cfg(feature = "wry-app")]
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

#[cfg(feature = "wry-app")]
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
