
mod simple_tests {
    use get_cookies::{read_cookie_until, read_cookie_with_options, CookieReadingOptions};
    
    #[tokio::test]
    async fn simple() -> Result<(), Box<dyn std::error::Error>> {
        let cookie_str = read_cookie_until("https://github.com", |c| c.contains("logged_in=yes")).await?;
        println!("your github cookie: {cookie_str}");
        Ok(())
    }

    #[tokio::test]
    async fn simple_with_options() -> Result<(), Box<dyn std::error::Error>> {
        let cookie_str = read_cookie_with_options("https://github.com", |c| c.contains("logged_in=yes"), 
            CookieReadingOptions {
                window_title: "Getting your cookie!".to_string(), // title of the popped-up window
                print_messages: false // disables any use of println by the library
            }).await?;
        println!("your github cookie: {cookie_str}");
        Ok(())
    }
}