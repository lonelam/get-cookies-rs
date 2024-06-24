mod read_cookie;
pub use read_cookie::*;

#[cfg(test)]
mod simple_tests {
    use crate::{read_cookie_until, read_cookie_with_title};

    #[tokio::test]
    async fn simple() -> Result<(), Box<dyn std::error::Error>> {
        let cookie_str = read_cookie_until("https://github.com", |c| c.contains("logged_in=yes")).await?;
        println!("your github cookie: {cookie_str}");
        Ok(())
    }

    #[tokio::test]
    async fn simple_with_title() -> Result<(), Box<dyn std::error::Error>> {
        let cookie_str = read_cookie_with_title("https://github.com", |c| c.contains("logged_in=yes"), "Getting your cookie!").await?;
        println!("your github cookie: {cookie_str}");
        Ok(())
    }
}