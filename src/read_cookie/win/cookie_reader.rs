pub struct CookieReadEvent {
    pub cookie_str: String,
}

use tao::event_loop::EventLoopProxy;
use tokio::{
    sync::mpsc::Sender,
    time::{self, Duration},
};

use webview2_com::{
    GetCookiesCompletedHandler, Microsoft::Web::WebView2::Win32::ICoreWebView2CookieManager,
};

use windows_core::{HSTRING, PWSTR};

pub async fn start_send_user_event_by_interval(event_loop_proxy: EventLoopProxy<CookieReadEvent>) {
    println!("Start ticking...");

    let mut interval = time::interval(Duration::from_secs(1));
    loop {
        interval.tick().await;
        let _ = event_loop_proxy.send_event(CookieReadEvent {
            cookie_str: String::from("hello"),
        });
    }
}

pub fn read_from_cookie_manager(
    cookie_manager: &ICoreWebView2CookieManager,
    current_host: &String,
    tx: &Sender<String>,
) {
    let current_host_name = HSTRING::from(current_host);
    let tx = tx.clone();
    let handler: webview2_com::Microsoft::Web::WebView2::Win32::ICoreWebView2GetCookiesCompletedHandler = GetCookiesCompletedHandler::create(Box::new(
                move |_, cookie_list| -> Result<(), windows_core::Error> {
                    unsafe {
                        let mut cookie_str = String::new();
                    if !cookie_list.is_none() {
                        let cookies = cookie_list.unwrap();
                        let mut cnt: u32 = 0;
                        cookies.Count(&mut cnt).unwrap();
                        for i in 0..cnt {
                            let cookie_item = cookies.GetValueAtIndex(i)?;
                            let mut name: PWSTR = PWSTR::null();
                            let mut value: PWSTR = PWSTR::null();
                            cookie_item.Name(&mut name)?;
                            cookie_item.Value(&mut value)?;
                            cookie_str.push_str(&format!("{}={}", name.to_string()?, value.to_string()?));
                            if i != cnt - 1 {
                                cookie_str.push_str(";");
                            }
                            // println!("{}: {}", name.to_string()?, value.to_string()?);
                        }
                    }
                    tx.blocking_send(cookie_str).unwrap();
                }
                Ok(())
            },
        ));

    // println!("Checking for cookies from {}", current_host);

    unsafe {
        cookie_manager
            .GetCookies(&current_host_name, &handler)
            .unwrap();
    }

    // println!("End of GetCookies call");
}
