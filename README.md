# get-cookies-rs
[![Crates.io](https://img.shields.io/crates/v/get-cookies.svg)](https://crates.io/crates/get-cookies)

English | [简体中文](README_zh.md)
`get-cookies-rs` is a Rust library designed to facilitate the retrieval of cookies from any website using the `Wry` library for easy cross-platform compatibility. This makes it ideal for developers needing a consistent tool across Windows, macOS, and Linux, without relying on specific browsers which may enlarge your program size.


## Features
- **Cross-Platform Support**: Thanks to `Wry`, `get-cookies-rs` operates seamlessly on various operating systems, making it an excellent choice for cross-platform applications.
- **Asynchronous API**: The library supports asynchronous operations, utilizing Rust's modern async/await syntax for efficient, non-blocking I/O.
- **Flexible Cookie Retrieval**: Allows users to retrieve cookies based on customizable conditions, offering robust solutions for web scraping and automated testing.

## Installation
To use `get-cookies-rs` in your project, add it to your `Cargo.toml` dependencies:

```toml
[dependencies]
get-cookies = "0.1.0"
```

## Prerequisites
Linux need extra browser gtklib installed, see [wry documentation](https://github.com/tauri-apps/wry/tree/wry-v0.39.3) to install.
Windows and MacOS need no extra dependency.


## Usage
```rust
use std::{str::FromStr, sync::Arc};

use get_cookies::read_cookie_until;
use regex::Regex;
use reqwest::header::{ACCEPT, COOKIE, ORIGIN, REFERER, USER_AGENT};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cookie_store = Arc::new(reqwest::cookie::Jar::default());
    let client = reqwest::Client::builder()
        .cookie_provider(cookie_store.clone())
        .build()?;
    let repo_url = "https://github.com/lonelam/get-cookies-rs";
    let cookie_str = read_cookie_until("https://github.com", |cookie_str: &String| {
        cookie_str.contains("logged_in=yes")
    })
    .await?;

    let cookies = cookie_str.split(';');
    for c_str in cookies {
        cookie_store.add_cookie_str(c_str, &reqwest::Url::from_str("https://github.com")?);
    }
    let repository_page = client.get(repo_url).send().await?;
    let html_content = repository_page.text().await?;
    let auth_re = Regex::new(
        r#"<form class="unstarred js-social-form" data-turbo="(.*)" action="(.*)/star" accept-charset="UTF-8" method="post"><input type="hidden" name="authenticity_token" value="(.*)" autocomplete="off" />"#,
    )?;
    let auth_token = auth_re
        .captures(&html_content)
        .map_or(String::new(), |r| r[3].to_string());

    let star_resp = client
        .post(format!("{}/star", repo_url))
        .header(REFERER, repo_url)
        .header(COOKIE, cookie_str)
        .header(ORIGIN, "https://github.com")
        .header(ACCEPT, "application/json")
        .multipart(
            reqwest::multipart::Form::new()
                .text("authenticity_token", auth_token)
                .text("context", "repository"),
        )
        .send()
        .await?;
    println!("{}", star_resp.text().await?);
    Ok(())
}


```

## Contributing
We welcome contributions to make **get-cookies-rs** even better! If you're interested in contributing, please fork the repository, commit your changes, and submit a pull request. For significant changes, please first open an issue to discuss what you would like to change.

## License
**get-cookies-rs** is distributed under the MIT License, which permits free use, modification, distribution, and private use of the software as long as copyright and license notices are preserved.