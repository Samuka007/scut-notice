use std::str::FromStr;

pub const UA: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";

pub struct Client {
    pub client: reqwest::Client,
}

impl Client {
    pub async fn new() -> Self {
        // Set up a more complete browser user agent
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::USER_AGENT,
            reqwest::header::HeaderValue::from_static(UA),
        );
        headers.insert(
            reqwest::header::ACCEPT,
            reqwest::header::HeaderValue::from_static(
                "application/json, text/javascript, */*; q=0.01",
            ),
        );
        headers.insert(
            reqwest::header::ACCEPT_LANGUAGE,
            reqwest::header::HeaderValue::from_static("en-US,en;q=0.9,zh-CN;q=0.8,zh;q=0.7"),
        );
        headers.insert(
            reqwest::header::CONTENT_TYPE,
            reqwest::header::HeaderValue::from_static(
                "application/x-www-form-urlencoded; charset=UTF-8",
            ),
        );
        headers.insert(
            reqwest::header::REFERER,
            reqwest::header::HeaderValue::from_static(
                "https://jw.scut.edu.cn/zhinan/cms/toPosts.do?category=0",
            ),
        );
        headers.insert(
            "X-Requested-With",
            reqwest::header::HeaderValue::from_static("XMLHttpRequest"),
        );
        headers.insert(
            "Origin",
            reqwest::header::HeaderValue::from_static("https://jw.scut.edu.cn"),
        );

        // First, visit the main page to get cookies
        let client = reqwest::ClientBuilder::new()
            .default_headers(headers)
            .cookie_store(true)
            .timeout(std::time::Duration::from_secs(30))
            .redirect(reqwest::redirect::Policy::limited(10))
            .build()
            .unwrap();

        println!("First establishing a session by visiting the main page...");
        let main_page_url = "https://jw.scut.edu.cn/zhinan/cms/toPosts.do?category=0";
        let main_page_resp = client.get(main_page_url).send().await.unwrap();

        if !main_page_resp.status().is_success() {
            eprintln!(
                "Failed to access the main page: HTTP {}",
                main_page_resp.status()
            );
        }

        Client { client }
    }
}

pub fn setup_logging() {
    let log_level = match std::env::var("RUST_LOG") {
        Ok(val) => val,
        Err(_) => "info".to_string(),
    };

    env_logger::Builder::new()
        .filter(None, log::LevelFilter::from_str(&log_level).unwrap())
        .init();
}
