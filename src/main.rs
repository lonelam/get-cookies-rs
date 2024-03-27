// use windows::Win32::System::Com::IUnknown;
use get_cookies::read_cookie;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cookie_str = read_cookie("https://juejin.cn").await?;
    println!("cookie_str: {}", cookie_str);

    Ok(())
}
