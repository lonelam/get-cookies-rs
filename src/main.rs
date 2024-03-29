// use windows::Win32::System::Com::IUnknown;
use get_cookies::read_cookie;
use get_cookies::read_cookie_until;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // let cookie_str = read_cookie("https://juejin.cn").await?;
    // println!("cookie_str: {}", cookie_str);

    let pattern = String::from("captcha_ticket_v2=");
    let cookie_str = read_cookie_until("https://zhihu.com", move |cookie_str: &String| {
        println!("cookie_str: {}", cookie_str);
        cookie_str.contains(&pattern)
    })
    .await?;

    println!("cookie_str: {}", cookie_str);
    Ok(())
}
