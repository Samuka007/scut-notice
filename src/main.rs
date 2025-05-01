use anyhow::Result;
use chrono::Local;
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;

#[derive(Debug, Serialize, Deserialize)]
struct Notice {
    title: String,
    url: String,
    date: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Set up the Chrome Windows user agent
    let chrome_ua = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/112.0.0.0 Safari/537.36";

    // Create a client with the specified user agent
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_static(chrome_ua));

    let client = reqwest::Client::builder()
        .default_headers(headers)
        .build()?;

    // Define the URL to scrape
    let url = "https://jw.scut.edu.cn/zhinan/cms/toPosts.do";

    // Send a GET request to the URL
    println!("Fetching data from {}", url);
    let response = client.get(url).send().await?;

    // Check if the request was successful
    if !response.status().is_success() {
        eprintln!("Failed to fetch data: HTTP {}", response.status());
        return Ok(());
    }

    // Get the HTML content
    let html_content = response.text().await?;

    // Parse the HTML
    let document = Html::parse_document(&html_content);

    // Define selectors for the notices
    // Note: These selectors might need adjustment based on the actual structure of the webpage
    let article_selector = Selector::parse(".list-item")
        .unwrap_or_else(|_| Selector::parse("div.news-list ul li").unwrap());

    let mut notices = Vec::new();

    // Extract the notices
    for article in document.select(&article_selector) {
        // Try different selectors for links and titles
        let link_selector = Selector::parse("a").unwrap();
        let title_selector = Selector::parse("a").unwrap();
        let date_selector =
            Selector::parse(".date").unwrap_or_else(|_| Selector::parse("span.time").unwrap());

        if let Some(link_element) = article.select(&link_selector).next() {
            let url = link_element.value().attr("href").unwrap_or("").to_string();
            let full_url = if url.starts_with("http") {
                url
            } else {
                format!("https://jw.scut.edu.cn{}", url)
            };

            let title = article
                .select(&title_selector)
                .next()
                .map_or("".to_string(), |el| {
                    el.text().collect::<Vec<_>>().join("").trim().to_string()
                });

            let date = article
                .select(&date_selector)
                .next()
                .map_or("".to_string(), |el| {
                    el.text().collect::<Vec<_>>().join("").trim().to_string()
                });

            if !title.is_empty() {
                notices.push(Notice {
                    title,
                    url: full_url,
                    date,
                });
            }
        }
    }

    // Check if we found any notices
    if notices.is_empty() {
        println!("No notices found. The webpage structure might have changed or there might be an issue with the selectors.");

        // Save the HTML content to a file for debugging
        let timestamp = Local::now().format("%Y%m%d_%H%M%S").to_string();
        let debug_file = format!("debug_html_{}.html", timestamp);
        let mut file = File::create(&debug_file)?;
        file.write_all(html_content.as_bytes())?;
        println!("Saved HTML content to {} for debugging", debug_file);
    } else {
        // Print the notices
        println!("Found {} notices:", notices.len());
        for (i, notice) in notices.iter().enumerate() {
            println!("{}. {} ({})", i + 1, notice.title, notice.date);
            println!("   URL: {}", notice.url);
        }

        // Save the notices to a JSON file
        let json = serde_json::to_string_pretty(&notices)?;
        let timestamp = Local::now().format("%Y%m%d_%H%M%S").to_string();
        let output_file = format!("scut_notices_{}.json", timestamp);
        let mut file = File::create(&output_file)?;
        file.write_all(json.as_bytes())?;
        println!("Notices saved to {}", output_file);
    }

    Ok(())
}
