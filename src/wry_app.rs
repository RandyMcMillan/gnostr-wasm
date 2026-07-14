use std::{
    error::Error,
    io,
    net::{SocketAddr, TcpStream},
    path::PathBuf,
    process::{Child, Command, Stdio},
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use muda::{Menu, MenuEvent, MenuItem, PredefinedMenuItem, Submenu};
use tao::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoopBuilder},
    window::{Window, WindowBuilder},
};
use url::Url;
#[cfg(any(
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "openbsd",
    target_os = "netbsd"
))]
use tao::platform::unix::WindowExtUnix;
#[cfg(target_os = "windows")]
use tao::platform::windows::{EventLoopBuilderExtWindows, WindowExtWindows};
#[cfg(any(
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "openbsd",
    target_os = "netbsd"
))]
use wry::WebViewBuilderExtUnix;
use wry::WebViewBuilder;

const HOST: &str = "127.0.0.1";
const PORT: u16 = 8080;

enum UserEvent {
    MenuEvent(MenuEvent),
}

pub fn run() -> Result<(), Box<dyn Error>> {
    let app_source = app_source()?;

    let mut event_loop_builder = EventLoopBuilder::<UserEvent>::with_user_event();

    let event_loop = event_loop_builder.build();
    let proxy = event_loop.create_proxy();
    MenuEvent::set_event_handler(Some(move |event| {
        let _ = proxy.send_event(UserEvent::MenuEvent(event));
    }));

    let menu_bar = Menu::new();
    let app_menu = Submenu::new("gnostr-wasm", true);
    let about_item = MenuItem::new("About gnostr-wasm", true, None);
    let quit_item = MenuItem::new("Quit", true, None);
    app_menu
        .append_items(&[
            &about_item,
            &PredefinedMenuItem::separator(),
            &quit_item,
        ])
        .unwrap();
    menu_bar.append(&app_menu).unwrap();

    let window = WindowBuilder::new()
        .with_title("gnostr-wasm")
        .build(&event_loop)?;

    #[cfg(target_os = "windows")]
    unsafe {
        menu_bar.init_for_hwnd(window.hwnd() as _).unwrap();
    }
    #[cfg(any(
        target_os = "linux",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "openbsd",
        target_os = "netbsd"
    ))]
    {
        menu_bar
            .init_for_gtk_window(window.gtk_window(), window.default_vbox())
            .unwrap();
    }
    #[cfg(target_os = "macos")]
    {
        menu_bar.init_for_nsapp();
    }

    let _webview = create_webview(&window)?.with_url(&app_source.url).build(&window)?;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => stop_server(&app_source.server, control_flow),
            Event::UserEvent(UserEvent::MenuEvent(event)) => {
                if event.id() == about_item.id() {
                    println!("gnostr-wasm: native About menu selected");
                } else if event.id() == quit_item.id() {
                    stop_server(&app_source.server, control_flow);
                }
            }
            _ => {}
        }
    });
}

fn stop_server(server: &Arc<Mutex<Option<Child>>>, control_flow: &mut ControlFlow) {
    if let Some(mut child) = server.lock().ok().and_then(|mut slot| slot.take()) {
        let _ = child.kill();
        let _ = child.wait();
    }
    *control_flow = ControlFlow::Exit;
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

struct AppSource {
    url: String,
    server: Arc<Mutex<Option<Child>>>,
}

fn app_source() -> Result<AppSource, Box<dyn Error>> {
    if let Some(index) = bundled_index_path()? {
        return Ok(AppSource {
            url: file_url(index),
            server: Arc::new(Mutex::new(None)),
        });
    }

    let server = Arc::new(Mutex::new(Some(spawn_trunk_serve()?)));
    wait_for_server()?;

    Ok(AppSource {
        url: format!("http://{HOST}:{PORT}"),
        server,
    })
}

fn bundled_index_path() -> Result<Option<PathBuf>, io::Error> {
    let exe = std::env::current_exe()?;
    let Some(contents_dir) = exe.parent().and_then(|p| p.parent()) else {
        return Ok(None);
    };
    let index = contents_dir.join("Resources").join("dist").join("index.html");
    Ok(index.exists().then_some(index))
}

fn file_url(path: PathBuf) -> String {
    Url::from_file_path(path)
        .expect("bundled index.html should convert to a file URL")
        .to_string()
}

#[cfg(any(
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "openbsd",
    target_os = "netbsd"
))]
fn create_webview(_window: &Window) -> Result<wry::WebViewBuilder<'_>, wry::Error> {
    Ok(WebViewBuilder::new_gtk(window.default_vbox().unwrap()))
}

#[cfg(not(any(
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "openbsd",
    target_os = "netbsd"
)))]
fn create_webview(_window: &Window) -> Result<wry::WebViewBuilder<'_>, wry::Error> {
    Ok(WebViewBuilder::new())
}
