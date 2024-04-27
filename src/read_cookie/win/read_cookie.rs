use std::borrow::Borrow;
use std::sync::Arc;
use tao::event_loop::EventLoopProxy;
use tao::platform::run_return::EventLoopExtRunReturn;
use tao::platform::windows::EventLoopBuilderExtWindows;
use tokio::sync::{mpsc, oneshot};
use webview2_com::Microsoft::Web::WebView2::Win32::ICoreWebView2_2;
// use windows::Win32::System::Com::IUnknown;
use super::cookie_reader::read_from_cookie_manager;
use super::cookie_reader::{start_send_user_event_by_interval, CookieReadEvent};
use windows_core::ComInterface;

use tao::{
    event::{Event, StartCause, WindowEvent},
    event_loop::{ControlFlow, EventLoopBuilder},
    window::WindowBuilder,
};
use wry::{WebView, WebViewBuilder, WebViewExtWindows};

pub async fn read_cookie_until<T: Fn(&String) -> bool>(
    target_url: &str,
    matcher: T,
) -> Result<String, Box<dyn std::error::Error>> {
    let domain_str = String::from(target_url);

    let (tx, mut rx) = mpsc::channel::<String>(32);
    let (event_loop_tx, event_loop_rx) = oneshot::channel::<EventLoopProxy<CookieReadEvent>>();

    let _ = std::thread::spawn(move || {
        let mut event_loop = EventLoopBuilder::<CookieReadEvent>::with_user_event()
            .with_any_thread(true)
            .build();
        let window = WindowBuilder::new()
            .with_title("Login And Wait For The Window Closed")
            .build(&event_loop)
            .unwrap();
        let webview: WebView = WebViewBuilder::new(&window)
            .with_incognito(true)
            .with_url(&domain_str)
            .expect("Failed to set URL")
            .build()
            .expect("Failed to build WebView");
        // let (tx, rx) = std::sync::mpsc::channel();
        let controller = Arc::new(webview.controller());

        let webview2 = unsafe { controller.CoreWebView2().unwrap() };

        let webview2_2: ICoreWebView2_2 = webview2.cast().unwrap();

        let cookie_manager = unsafe { webview2_2.CookieManager().unwrap() };
        let event_loop_proxy = event_loop.create_proxy();
        event_loop_tx.send(event_loop_proxy).unwrap();

        let _ = event_loop.run_return(move |event, _, control_flow| {
            *control_flow = ControlFlow::Wait;

            match event {
                Event::NewEvents(StartCause::Init) => println!("Wry has started!"),
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => *control_flow = ControlFlow::Exit,
                Event::UserEvent(_) => {
                    // println!("User event received");
                    read_from_cookie_manager(&cookie_manager, &domain_str, &tx);
                }
                _ => (),
            }
        });
    });

    let event_loop_proxy = event_loop_rx.await?;
    // println!("Start spawning");
    let _ = tokio::spawn(start_send_user_event_by_interval(event_loop_proxy));
    loop {
        let cookie_str = rx.recv().await;
        if cookie_str.is_none() {
            // println!("cookie_str is none");
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            continue;
        }
        let cookie_str = cookie_str.unwrap();
        // println!("cookie_str: {}", cookie_str);
        if matcher(&cookie_str) {
            return Ok(cookie_str);
        }
    }
}

pub async fn read_cookie(target_url: &str) -> Result<String, Box<dyn std::error::Error>> {
    read_cookie_until(target_url, |latest_cookie_str| {
        latest_cookie_str.contains("sessionid")
    })
    .await
}
