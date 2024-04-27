use super::user_event::{CookieReadEvent, EventType};
use std::error::Error;
use std::sync::{Arc, Mutex};
use tao::event::{Event, StartCause, WindowEvent};
use tao::event_loop::{ControlFlow, EventLoopProxy};
use tao::platform::run_return::EventLoopExtRunReturn;
use tao::{event_loop::EventLoopBuilder, window::WindowBuilder};
use wry::{WebView, WebViewBuilder, WebViewExtUnix};
use webkit2gtk::{ CookieManagerExt, WebViewExt, WebsiteDataManagerExt};

fn start_send_user_event_by_interval(
    event_loop_proxy: EventLoopProxy<CookieReadEvent>,
    cookie_returned: Arc<Mutex<Option<String>>>,
) -> std::thread::JoinHandle<()> {
    println!("Start ticking...");
    // Set the interval duration
    let interval = std::time::Duration::from_secs(1);

    // Create a new thread for the interval timer
    let handle: std::thread::JoinHandle<()> = std::thread::spawn(move || {
        let mut next_tick = std::time::Instant::now() + interval;
        loop {
            // Wait until the next tick time
            let now = std::time::Instant::now();
            if now < next_tick {
                std::thread::sleep(next_tick - now);
            }

            if cookie_returned.lock().unwrap().is_some() {
                break;
            }
            // Calculate the next tick time
            next_tick += interval;

            let _ = event_loop_proxy.send_event(CookieReadEvent {
                m_type: EventType::CookieRead,
            });
        }
    });
    handle
}

pub fn read_cookie_until_sync<T: Fn(&String) -> bool + Send + 'static>(
    target_url: &str,
    matcher: T,
) -> Result<String, Box<dyn std::error::Error>> {
    let domain_str = String::from(target_url);

    let pattern_matcher = Box::new(matcher);
    let returned_cookie: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
    let p_returned_cookie_for_completion_handler = returned_cookie.clone();
    let p_returned_cookie_for_timer = returned_cookie.clone();

    let mut event_loop = EventLoopBuilder::<CookieReadEvent>::with_user_event().build();
    let window = WindowBuilder::new()
        .with_title("Login And Wait For The Window Closed")
        .build(&event_loop)
        .unwrap();
    let webview: WebView = WebViewBuilder::new(&window)
        .with_incognito(true)
        .with_url(target_url)
        .expect("Failed to set URL")
        .build()
        .expect("Failed to build WebView");
    let event_loop_proxy = event_loop.create_proxy();
    // event_loop_tx.send(event_loop_proxy).unwrap();
    let _ = start_send_user_event_by_interval(event_loop_proxy, p_returned_cookie_for_timer);
    let event_loop_proxy = event_loop.create_proxy();
    // let completion_handler = block::ConcreteBlock::new(move |http_cookies: cocoa::base::id| {
    //     // println!("Cookie has read");
    //     let cookie_cnt = unsafe { NSArray::count(http_cookies) };
    //     // println!("There're {} in arr", cookie_cnt);
    //     let mut cookie_str = String::new();
    //     for i in 0..cookie_cnt {
    //         let cookie = unsafe { NSArray::objectAtIndex(http_cookies, i) };
    //         let cookie_name = unsafe {
    //             let name: cocoa::base::id = msg_send![cookie, name];
    //             let name_str = std::ffi::CStr::from_ptr(name.UTF8String())
    //                 .to_str()
    //                 .unwrap();
    //             name_str
    //         };
    //         let cookie_value = unsafe {
    //             let value: cocoa::base::id = msg_send![cookie, value];
    //             let value_str = std::ffi::CStr::from_ptr(value.UTF8String())
    //                 .to_str()
    //                 .unwrap();
    //             value_str
    //         };
    //         cookie_str += cookie_name;
    //         cookie_str += "=";
    //         cookie_str += cookie_value;
    //         if i != cookie_cnt - 1 {
    //             cookie_str += ";";
    //         }
    //         // println!("The current cookie is {}={}", cookie_name, cookie_value);
    //     }
    //     // #[cfg(debug_assertions)]
    //     // println!("The current cookies is: {}", cookie_str);
    //     if pattern_matcher(&cookie_str) {
    //         let mut p_cookie = p_returned_cookie_for_completion_handler.lock().unwrap();
    //         *p_cookie = Some(cookie_str);
    //         let _ = event_loop_proxy.send_event(CookieReadEvent {
    //             m_type: EventType::Finish,
    //         });
    //     }
    // })
    // .copy();
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
                            let webview = webview.webview();
                            let data_manager = webview.website_data_manager().unwrap();
                            let cookie_manager = data_manager.cookie_manager().unwrap();
                            cookie_manager.cookies(&domain_str,  None::<&webkit2gtk::gio::Cancellable>, move |result:Result<Vec<soup::Cookie>, webkit2gtk::Error> | {
                                println!("Cookie read finished");
                            });
                        }
                    }
                    EventType::Finish => *control_flow = ControlFlow::Exit,
                }
            }
            _ => (),
        }
    });
    let mut p_latest_cookie = returned_cookie.lock().unwrap();
    if p_latest_cookie.is_none() {
        // mannualy close the window.
        *p_latest_cookie = Some(String::new());
    }
    Ok(p_latest_cookie.clone().unwrap())
}

pub async fn read_cookie_until<T: Fn(&String) -> bool + Send + 'static>(
    target_url: &str,
    matcher: T,
) -> Result<String, Box<dyn std::error::Error>> {
    read_cookie_until_sync(target_url, matcher)
}
