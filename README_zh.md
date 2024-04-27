# get-cookies-rs
[![Crates.io](https://img.shields.io/crates/v/get-cookies.svg)](https://crates.io/crates/get-cookies)

[English](README.md) | [简体中文](README_zh.md)
`get-cookies-rs` 是一个 Rust 库，设计用于使用 `Wry` 库从任何网站检索 cookies，以便轻松实现跨平台兼容性。这使它成为需要在 Windows、macOS 和 Linux 上使用一致工具的开发者的理想选择，而无需依赖可能会增大程序大小的特定浏览器。

## 特点
- **跨平台支持**：得益于 `Wry`，`get-cookies-rs` 可以无缝地在各种操作系统上运行，是跨平台应用程序的绝佳选择。
- **异步 API**：库支持异步操作，使用 Rust 的现代 async/await 语法实现高效的非阻塞 I/O。
- **灵活的 Cookie 检索**：允许用户基于可定制的条件检索 cookies，为网络抓取和自动化测试提供强大的解决方案。

## 安装
要在您的项目中使用 `get-cookies-rs`，请将其添加到您的 Cargo.toml 依赖项中：


```toml
[dependencies]
get-cookies = "0.1.0"
```

## Prerequisites
Linux need extra browser gtklib installed, see [wry documentation](https://github.com/tauri-apps/wry/tree/wry-v0.39.3) to install.
Windows and MacOS need no extra dependency.


## 使用方法

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
        r#"<form class="unstarred js-social-form" data-turbo="(.*)" action="(.*)/star" accept-charset="UTF-8" method="post"><input type="hidden" name="authenticity_token" value="(.*)" autocomplete="off" />"#
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

## 贡献
我们欢迎社区的贡献来使 **get-cookies-rs** 更好！如果您有兴趣贡献，请 fork 该仓库，提交您的更改，并提交一个 pull request。对于重大更改，请先开一个 issue 讨论您希望改变的内容。

## 许可证
**get-cookies-rs** 根据 MIT 许可证发布，该许可证允许自由使用、修改、分发和私人使用软件，只要保留版权和许可声明。
