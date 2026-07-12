use tao::{
  dpi::{LogicalPosition, LogicalSize},
  event::{Event, WindowEvent},
  event_loop::{ControlFlow, EventLoop, EventLoopBuilder, EventLoopProxy},
  window::{Window, WindowBuilder},
};
use wry::{BackgroundThrottlingPolicy, PageLoadEvent, Rect, WebView, WebViewBuilder};
use std::{thread, time::Duration};

#[cfg(windows)]
use wry::ScrollBarStyle;

const DEFAULT_URL: &str = "http://gnostr.cloud";
const MOBILE_USER_AGENT: &str =
  "Mozilla/5.0 (iPhone; CPU iPhone OS 17_5 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.5 Mobile/15E148 Safari/604.1";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ViewRole {
  Primary,
  Secondary,
}

#[derive(Debug, Clone)]
enum AppEvent {
  Navigate(String),
  Back,
  Forward,
  Reload,
  RefreshScrollbars,
  PageLoaded { role: ViewRole },
}

fn other_role(role: ViewRole) -> ViewRole {
  match role {
    ViewRole::Primary => ViewRole::Secondary,
    ViewRole::Secondary => ViewRole::Primary,
  }
}

fn active_webview<'a>(role: ViewRole, primary: &'a WebView, secondary: &'a WebView) -> &'a WebView {
  match role {
    ViewRole::Primary => primary,
    ViewRole::Secondary => secondary,
  }
}

fn rehide_scrollbars(webview: &WebView) {
  let _ = webview.evaluate_script("window.__wryHideScrollbars && window.__wryHideScrollbars();");
}

fn full_bounds(window: &Window) -> Rect {
  let size = window.inner_size();
  Rect {
    position: LogicalPosition::new(0.0, 0.0).into(),
    size: LogicalSize::new(size.width as f64, size.height as f64).into(),
  }
}

fn browser_init_script(chrome: &str, auto_redirect: bool) -> String {
  let auto_redirect = if auto_redirect { "true" } else { "false" };
  format!(
    r#"
(() => {{
  if (window.__wryBrowserUiInstalled) return;
  window.__wryBrowserUiInstalled = true;

  const chromeHtml = {chrome:?};
  const autoRedirect = {auto_redirect};
  const defaultUrl = {default_url:?};

  const mount = () => {{
    if (document.getElementById('wry-browser-host')) return;

    const host = document.createElement('div');
    host.id = 'wry-browser-host';
    host.innerHTML = chromeHtml;
    document.documentElement.appendChild(host);

    const style = host.querySelector('style');
    if (style && document.head) {{
      document.head.appendChild(style);
    }}

    const ensureScrollbarStyles = () => {{
      if (!document.head) return;

      let scrollbarStyle = document.getElementById('wry-scrollbar-style');
      if (!scrollbarStyle) {{
        scrollbarStyle = document.createElement('style');
        scrollbarStyle.id = 'wry-scrollbar-style';
        document.head.appendChild(scrollbarStyle);
      }}

      scrollbarStyle.textContent = `
        html, body, #wry-browser-host, #wry-browser-host * {{
          -ms-overflow-style: none !important;
          scrollbar-width: none !important;
        }}
        html, body {{
          overflow-y: auto !important;
          overflow-x: hidden !important;
        }}
        html::-webkit-scrollbar,
        body::-webkit-scrollbar,
        #wry-browser-host::-webkit-scrollbar,
        #wry-browser-host *::-webkit-scrollbar,
        *::-webkit-scrollbar {{
          width: 0 !important;
          height: 0 !important;
          display: none !important;
        }}
      `;
    }};

    const rehideScrollbars = () => {{
      ensureScrollbarStyles();
      const root = document.documentElement;
      root.style.setProperty('-ms-overflow-style', 'none', 'important');
      root.style.setProperty('scrollbar-width', 'none', 'important');
      root.style.setProperty('overflow-y', 'auto', 'important');
      root.style.setProperty('overflow-x', 'hidden', 'important');
      if (document.body) {{
        document.body.style.setProperty('-ms-overflow-style', 'none', 'important');
        document.body.style.setProperty('scrollbar-width', 'none', 'important');
        document.body.style.setProperty('overflow-y', 'auto', 'important');
        document.body.style.setProperty('overflow-x', 'hidden', 'important');
      }}
    }};

    const guardScrollbars = () => {{
      rehideScrollbars();
      window.requestAnimationFrame(rehideScrollbars);
      window.setTimeout(rehideScrollbars, 0);
    }};
    window.__wryHideScrollbars = guardScrollbars;

    const applyChromeOffset = () => {{
      const rect = host.getBoundingClientRect();
      const offset = Math.ceil(rect.bottom + 12);
      const footerRect = footer?.getBoundingClientRect();
      const footerOffset = footerRect ? Math.ceil(footerRect.height + 14) : 42;
      const root = document.documentElement;
      root.style.setProperty('--wry-browser-offset', `${{offset}}px`);
      root.style.setProperty('--wry-browser-footer-offset', `${{footerOffset}}px`);
      root.style.setProperty('margin', '0', 'important');
      root.style.setProperty('padding-top', `${{offset}}px`, 'important');
      root.style.setProperty('padding-bottom', `${{footerOffset}}px`, 'important');
      root.style.setProperty('scroll-padding-top', `${{offset}}px`);
      root.style.setProperty('scroll-padding-bottom', `${{footerOffset}}px`);
      root.style.setProperty('-ms-overflow-style', 'none', 'important');
      root.style.setProperty('scrollbar-width', 'none', 'important');
      if (document.body) {{
        document.body.style.setProperty('margin', '0', 'important');
        document.body.style.setProperty('padding-top', '0', 'important');
        document.body.style.setProperty('padding-bottom', `${{footerOffset}}px`, 'important');
        document.body.style.setProperty('-ms-overflow-style', 'none', 'important');
        document.body.style.setProperty('scrollbar-width', 'none', 'important');
      }}
    }};

    const shell = host.querySelector('#wry-browser-ui');
    const footer = host.querySelector('#wry-browser-footer');
    const address = shell?.querySelector('[data-address]');
    const dot = footer?.querySelector('[data-dot]');
    const status = footer?.querySelector('[data-status]');
    const back = shell?.querySelector('[data-back]');
    const forward = shell?.querySelector('[data-forward]');
    const reload = shell?.querySelector('[data-reload]');
    const go = shell?.querySelector('[data-go]');

    const normalizeUrl = (raw) => {{
      const value = String(raw ?? '').trim();
      if (!value) return 'https://example.com';
      if (/^https?:\/\//i.test(value) || /^file:\/\//i.test(value)) return value;
      if (value.includes('.') && !value.includes(' ')) return `https://${{value}}`;
      return `https://duckduckgo.com/?q=${{encodeURIComponent(value)}}`;
    }};

    const setFooterState = (state, label) => {{
      if (dot) dot.dataset.state = state;
      if (status) status.textContent = label;
    }};

    const currentDomain = () => {{
      try {{
        return new URL(window.location.href).hostname || 'page';
      }} catch {{
        return 'page';
      }}
    }};

    const updateState = () => {{
      if (address) address.value = window.location.href;
      if (!navigator.onLine) {{
        setFooterState('offline', 'Offline');
        return;
      }}
      setFooterState('ready', `Viewing ${{currentDomain()}}`);
    }};

    const sendAction = (cmd, value = '') => {{
      if (window.ipc && typeof window.ipc.postMessage === 'function') {{
        window.ipc.postMessage([cmd, value].join('|'));
        return true;
      }}
      return false;
    }};

    const navigate = (raw) => {{
      const url = normalizeUrl(raw);
      guardScrollbars();
      if (address) address.value = url;
      setFooterState('loading', `Loading ${{(() => {{ try {{ return new URL(url).hostname || 'page'; }} catch {{ return 'page'; }} }})()}}`);
      if (!sendAction('navigate', url)) {{
        window.location.href = url;
      }}
    }};

    back?.addEventListener('click', () => {{
      guardScrollbars();
      setFooterState('loading', `Loading ${{currentDomain()}}`);
      if (!sendAction('back')) {{
        history.back();
      }}
    }});
    forward?.addEventListener('click', () => {{
      guardScrollbars();
      setFooterState('loading', `Loading ${{currentDomain()}}`);
      if (!sendAction('forward')) {{
        history.forward();
      }}
    }});
    reload?.addEventListener('click', () => {{
      guardScrollbars();
      setFooterState('loading', `Reloading ${{currentDomain()}}`);
      if (!sendAction('reload')) {{
        window.location.reload();
      }}
    }});
    go?.addEventListener('click', () => navigate(address?.value));
    address?.addEventListener('keydown', (event) => {{
      if (event.key === 'Enter') navigate(address.value);
    }});

    shell?.querySelectorAll('[data-url]').forEach((node) => {{
      node.addEventListener('click', (event) => {{
        event.preventDefault();
        navigate(node.dataset.url);
      }});
    }});

    window.addEventListener('keydown', (event) => {{
      if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === 'l') {{
        event.preventDefault();
        address?.focus();
        address?.select();
      }}
      if (event.altKey && event.key === 'ArrowLeft') {{
        guardScrollbars();
        setFooterState('loading', `Loading ${{currentDomain()}}`);
        if (!sendAction('back')) {{
          history.back();
        }}
      }}
      if (event.altKey && event.key === 'ArrowRight') {{
        guardScrollbars();
        setFooterState('loading', `Loading ${{currentDomain()}}`);
        if (!sendAction('forward')) {{
          history.forward();
        }}
      }}
    }});

    window.addEventListener('pageshow', () => {{
      guardScrollbars();
      updateState();
    }});
    window.addEventListener('load', () => {{
      guardScrollbars();
      updateState();
    }});
    window.addEventListener('online', updateState);
    window.addEventListener('offline', () => setFooterState('offline', 'Offline'));
    window.addEventListener('scroll', guardScrollbars, {{ passive: true }});
    window.addEventListener('focus', guardScrollbars);
    window.addEventListener('visibilitychange', guardScrollbars);
    window.addEventListener('hashchange', guardScrollbars);
    window.addEventListener('popstate', guardScrollbars);
    window.addEventListener('resize', applyChromeOffset);
    if (window.ResizeObserver) {{
      const observer = new ResizeObserver(applyChromeOffset);
      observer.observe(host);
    }}
    if (window.MutationObserver) {{
      const scrollbarObserver = new MutationObserver(() => {{
        const root = document.documentElement;
        root.style.setProperty('-ms-overflow-style', 'none', 'important');
        root.style.setProperty('scrollbar-width', 'none', 'important');
        if (document.body) {{
          document.body.style.setProperty('-ms-overflow-style', 'none', 'important');
          document.body.style.setProperty('scrollbar-width', 'none', 'important');
        }}
      }});
      scrollbarObserver.observe(document.documentElement, {{
        attributes: true,
        childList: true,
        subtree: true,
      }});
    }}
    guardScrollbars();
    const scrollbarLoop = () => {{
      guardScrollbars();
      window.requestAnimationFrame(scrollbarLoop);
    }};
    window.requestAnimationFrame(scrollbarLoop);
    applyChromeOffset();
    updateState();
  }};

  if (document.readyState === 'loading') {{
    document.addEventListener('DOMContentLoaded', mount, {{ once: true }});
  }} else {{
    mount();
  }}

  if (autoRedirect && window.location.href === 'about:blank') {{
    setTimeout(() => {{
      window.location.replace(defaultUrl);
    }}, 0);
  }}
}})();
"#,
    chrome = chrome,
    default_url = DEFAULT_URL
  )
}

fn build_webview(
  window: &Window,
  proxy: EventLoopProxy<AppEvent>,
  role: ViewRole,
  visible: bool,
  script: String,
  bounds: Rect,
) -> wry::Result<WebView> {
  let load_proxy = proxy.clone();
  let ipc_proxy = proxy;

  let builder = WebViewBuilder::new()
    .with_initialization_script(script)
    .with_user_agent(MOBILE_USER_AGENT)
    .with_url("about:blank")
    .with_visible(visible)
    .with_focused(visible)
    .with_bounds(bounds)
    .with_background_throttling(BackgroundThrottlingPolicy::Disabled);

  #[cfg(windows)]
  let builder = builder.with_scroll_bar_style(ScrollBarStyle::FluentOverlay);
  #[cfg(not(windows))]
  let builder = builder;

  builder
    .with_ipc_handler(move |request| {
      let body = request.body().as_str();
      let mut parts = body.splitn(2, '|');
      let cmd = parts.next().unwrap_or_default();
      let value = parts.next().unwrap_or_default().to_string();

      let event = match cmd {
        "navigate" => Some(AppEvent::Navigate(value)),
        "back" => Some(AppEvent::Back),
        "forward" => Some(AppEvent::Forward),
        "reload" => Some(AppEvent::Reload),
        "refresh-scrollbars" => Some(AppEvent::RefreshScrollbars),
        _ => None,
      };

      if let Some(event) = event {
        let _ = ipc_proxy.send_event(event);
      }
    })
    .with_on_page_load_handler(move |event, _url| {
      if matches!(event, PageLoadEvent::Finished) {
        let _ = load_proxy.send_event(AppEvent::PageLoaded { role });
      }
    })
    .build_as_child(window)
}

fn main() -> wry::Result<()> {
  let event_loop: EventLoop<AppEvent> = EventLoopBuilder::with_user_event().build();
  let proxy = event_loop.create_proxy();
  let window = WindowBuilder::new()
    .with_title("gnostr.cloud")
    .build(&event_loop)
    .unwrap();

  let chrome = include_str!("gnostr-cloud.html");
  let active_script = browser_init_script(chrome, true);
  let preload_script = browser_init_script(chrome, false);
  let bounds = full_bounds(&window);

  let primary = build_webview(
    &window,
    proxy.clone(),
    ViewRole::Primary,
    true,
    active_script,
    bounds,
  )?;
  let secondary = build_webview(
    &window,
    proxy,
    ViewRole::Secondary,
    false,
    preload_script,
    bounds,
  )?;

  let mut active_role = ViewRole::Primary;
  let mut pending_preload: Option<ViewRole> = None;

  {
    let proxy = event_loop.create_proxy();
    thread::spawn(move || loop {
      thread::sleep(Duration::from_secs(3));
      let _ = proxy.send_event(AppEvent::RefreshScrollbars);
    });
  }

  #[allow(unreachable_code)]
  {
    event_loop.run(move |event, _, control_flow| {
      *control_flow = ControlFlow::Wait;

      match event {
        Event::UserEvent(AppEvent::Navigate(url)) => {
          let standby_role = other_role(active_role);
          let standby = active_webview(standby_role, &primary, &secondary);
          pending_preload = Some(standby_role);
          if let Err(err) = standby.load_url(&url) {
            eprintln!("failed to preload {url}: {err}");
            pending_preload = None;
          }
        }
        Event::UserEvent(AppEvent::Back) => {
          pending_preload = None;
          let _ =
            active_webview(active_role, &primary, &secondary).evaluate_script("history.back();");
        }
        Event::UserEvent(AppEvent::Forward) => {
          pending_preload = None;
          let _ = active_webview(active_role, &primary, &secondary)
            .evaluate_script("history.forward();");
        }
        Event::UserEvent(AppEvent::Reload) => {
          pending_preload = None;
          let _ = active_webview(active_role, &primary, &secondary).reload();
        }
        Event::UserEvent(AppEvent::RefreshScrollbars) => {
          rehide_scrollbars(&primary);
          rehide_scrollbars(&secondary);
        }
        Event::UserEvent(AppEvent::PageLoaded { role }) => {
          if pending_preload.is_some_and(|pending_role| pending_role == role) {
            let visible = active_webview(role, &primary, &secondary);
            let hidden = active_webview(other_role(role), &primary, &secondary);
            let _ = visible.evaluate_script("window.__wryHideScrollbars && window.__wryHideScrollbars();");
            let _ = visible.set_visible(true);
            let _ = hidden.set_visible(false);
            let _ = visible.focus();
            active_role = role;
            pending_preload = None;
          }
        }
        Event::WindowEvent {
          event: WindowEvent::Resized(_) | WindowEvent::ScaleFactorChanged { .. },
          ..
        } => {
          let bounds = full_bounds(&window);
          let _ = primary.set_bounds(bounds);
          let _ = secondary.set_bounds(bounds);
        }
        Event::WindowEvent {
          event: WindowEvent::CloseRequested,
          ..
        } => {
          *control_flow = ControlFlow::Exit;
        }
        _ => {}
      }
    });

    Ok(())
  }
}
