pub mod read_cookie {
    pub async fn read_cookie_until<T: Fn(&String) -> bool>(
        target_url: &str,
        matcher: T,
    ) -> Result<String, Box<dyn std::error::Error>> {
        Ok(String::from("Currently not supported"))
    }
}
