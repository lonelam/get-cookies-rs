# get-cookies-rs
A rust project to get cookies from any website by a simple call, powered by Wry.

## Usage
```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // let cookie_str = read_cookie("https://juejin.cn").await?;
    // println!("cookie_str: {}", cookie_str);

    let pattern = String::from("captcha_ticket_v2=");
    let cookie_str = read_cookie_until("https://github.com", move |cookie_str: &String| {
        println!("cookie_str: {}", cookie_str);
        cookie_str.contains(&pattern)
    })
    .await?;

    println!("get matched cookie_str success: {}", cookie_str);
    Ok(())
}
```