use std::fs::File;
use std::io::Write;
use std::str::FromStr;
use std::sync::Arc;

// use windows::Win32::System::Com::IUnknown;
use get_cookies::read_cookie_until;
use regex::Regex;
use reqwest;
use reqwest::cookie::CookieStore;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // let cookie_str = read_cookie("https://juejin.cn").await?;
    // println!("cookie_str: {}", cookie_str);

    let cookie_store = Arc::new(reqwest::cookie::Jar::default());
    let client = reqwest::Client::builder()
        // .cookie_store(true)
        .cookie_provider(cookie_store.clone())
        .build()?;

    let repo_url = "https://github.com/lonelam/get-cookies-rs";

    let pattern = String::from("logged_in=yes");
    let cookie_str = read_cookie_until("https://github.com", move |cookie_str: &String| {
        // println!("matching cookie_str from: {}", cookie_str);
        cookie_str.contains(&pattern)
    })
    .await?;

    println!("get matched cookie_str success.");

    let cookies = cookie_str.split(';');
    for c_str in cookies {
        cookie_store.add_cookie_str(c_str, &reqwest::Url::from_str("https://github.com")?);
    }

    let repository_page = client.get(repo_url).send().await?;
    let html_content = repository_page.text().await?;

    let mut file = File::create("index.html").expect("Could not create file");

    file.write_all(html_content.as_bytes())
        .expect("Failed to write data");

    let auth_re = Regex::new(
        r#"<form class="unstarred js-social-form" data-turbo="(.*)" action="(.*)/star" accept-charset="UTF-8" method="post"><input type="hidden" name="authenticity_token" value="(.*)" autocomplete="off" />"#,
    )?;

    let match_result = auth_re.captures(&html_content);

    if match_result.is_none() {
        println!("match_result is none");
        return Ok(());
    }

    let auth_token = match_result.map_or(String::new(), |r| r[3].to_string());

    // println!("cookies_before_star: {:?}", cookies_before_star);
    let cookies_before_star = cookie_store
        .cookies(&reqwest::Url::from_str(&format!("{}/star", repo_url))?)
        .unwrap();
    // println!("cookies_before_star: {:?}", cookies_before_star);
    let star_req = client
        .post(format!("{}/star", repo_url))
        .header(reqwest::header::REFERER, repo_url)
        .header(reqwest::header::COOKIE, cookies_before_star)
        .header(reqwest::header::ORIGIN, "https://github.com")
        .header(reqwest::header::USER_AGENT, "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.77 Safari/537.36")
        .header(reqwest::header::ACCEPT, "application/json")
        .multipart(
            reqwest::multipart::Form::new()
                .text("authenticity_token", auth_token)
                .text("context", String::from("repository")),
        )
        .build()?;

    // println!("star_req: {:?}", star_req);

    let resp = client.execute(star_req).await?;
    println!("resp: {:?}", resp.text().await?);

    Ok(())
}
