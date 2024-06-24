use std::{str::FromStr, sync::Arc};

use get_cookies::read_cookie_until;
use regex::Regex;
use reqwest::header::{ACCEPT, COOKIE, ORIGIN, REFERER};

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
