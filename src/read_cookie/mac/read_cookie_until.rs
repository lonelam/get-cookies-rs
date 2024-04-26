use super::user_event::{CookieReadEvent, EventType};
use cocoa::foundation::{NSArray, NSString};
use objc::{msg_send, sel, sel_impl};
use std::error::Error;
use std::process::exit;
use std::sync::Arc;
use tao::event::{Event, StartCause, WindowEvent};
use tao::event_loop::{ControlFlow, EventLoopProxy};
use tao::platform::macos::EventLoopWindowTargetExtMacOS;
use tao::platform::run_return::EventLoopExtRunReturn;
use tao::{event_loop::EventLoopBuilder, window::WindowBuilder};
use tokio::time;
use tokio::time::Duration;
use wry::{WebView, WebViewBuilder, WebViewExtMacOS};

async fn start_send_user_event_by_interval(event_loop_proxy: EventLoopProxy<CookieReadEvent>) {
    println!("Start ticking...");

    let mut interval = time::interval(Duration::from_secs(1));
    loop {
        interval.tick().await;
        let _ = event_loop_proxy.send_event(CookieReadEvent {
            m_type: EventType::CookieRead,
        });
    }
}

pub async fn read_cookie_until(
    target_url: &str,
    matcher: Arc<dyn Fn(&String) -> bool>,
) -> Result<String, Box<dyn std::error::Error>> {
    let domain_str = String::from(target_url);

    let pattern_matcher = Box::new(matcher);
    let (tx, mut rx) = tokio::sync::mpsc::channel::<String>(32);
    let (event_loop_tx, event_loop_rx) =
        tokio::sync::oneshot::channel::<EventLoopProxy<CookieReadEvent>>();

    let mut event_loop = EventLoopBuilder::<CookieReadEvent>::with_user_event().build();
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
    let event_loop_proxy = event_loop.create_proxy();
    event_loop_tx.send(event_loop_proxy).unwrap();
    let event_loop_proxy = event_loop.create_proxy();
    let completion_handler = block::ConcreteBlock::new(move |http_cookies: cocoa::base::id| {
        // println!("Cookie has read");
        let cookie_cnt = unsafe { NSArray::count(http_cookies) };
        // println!("There're {} in arr", cookie_cnt);
        let mut cookie_str = String::new();
        for i in 0..cookie_cnt {
            let cookie = unsafe { NSArray::objectAtIndex(http_cookies, i) };
            let cookie_name = unsafe {
                let name: cocoa::base::id = msg_send![cookie, name];
                let name_str = std::ffi::CStr::from_ptr(name.UTF8String())
                    .to_str()
                    .unwrap();
                name_str
            };
            let cookie_value = unsafe {
                let value: cocoa::base::id = msg_send![cookie, value];
                let value_str = std::ffi::CStr::from_ptr(value.UTF8String())
                    .to_str()
                    .unwrap();
                value_str
            };
            cookie_str += cookie_name;
            cookie_str += "=";
            cookie_str += cookie_value;
            if i != cookie_cnt - 1 {
                cookie_str += ";";
            }
            // println!("The current cookie is {}={}", cookie_name, cookie_value);
        }
        if pattern_matcher(&cookie_str) {
            tx.send(cookie_str);
            let _ = event_loop_proxy.send_event(CookieReadEvent {
                m_type: EventType::Finish,
            });
        }
    })
    .copy();
    event_loop.run_return(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::NewEvents(StartCause::Init) => println!("Wry has started!"),
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            Event::UserEvent(evt) => {
                match evt.m_type {
                    EventType::CookieRead => {
                        unsafe {
                            let webview: cocoa::base::id = webview.webview();

                            let configuration: cocoa::base::id = msg_send![webview, configuration];
                            let website_data_store: cocoa::base::id =
                                msg_send![configuration, websiteDataStore];

                            let http_cookie_store: cocoa::base::id =
                                msg_send![website_data_store, httpCookieStore];
                            // let block = completion_handler.copy();
                            let _: () =
                                msg_send![http_cookie_store, getAllCookies: &*completion_handler];
                        }
                    }
                    EventType::Finish => *control_flow = ControlFlow::Exit,
                }
            }
            _ => (),
        }
    });
    let result_cookie = rx.recv().await;
    if result_cookie.is_none() {
        Ok(String::from(""))
    } else {
        Ok(result_cookie.unwrap())
    }
}
