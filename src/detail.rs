use crate::{metadata::NoticeMetadata, util::Client};
use scraper::{ElementRef, Html, Selector};
use url::Url;

#[derive(Debug)]
pub struct NoticeDetail {
    pub metadata: NoticeMetadata,
    pub content: String,
    pub attachments: Vec<Attachment>,
}

#[derive(Debug)]
pub struct Attachment {
    pub name: String,
    pub url: Url,
}

impl Client {
    pub async fn fetch_notice_detail(&self, notice_id: NoticeMetadata) -> NoticeDetail {
        let url = format!(
            "https://jw.scut.edu.cn/zhinan/cms/article/view.do?type=posts&id={}",
            notice_id.id
        );

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|err| {
                eprintln!("Failed to fetch notice detail: {}", err);
                err
            })
            .unwrap();

        let html_content = response
            .text()
            .await
            .map_err(|err| {
                eprintln!("Failed to read response text: {}", err);
                err
            })
            .unwrap();

        // Parse HTML and extract content div
        let document = Html::parse_document(&html_content);
        let content_selector = Selector::parse("div.content").unwrap();

        // Extract and clean content
        let content = if let Some(content_div) = document.select(&content_selector).next() {
            self.extract_clean_content(&content_div)
        } else {
            log::error!("Could not find content div in the HTML");
            String::from("Content not found")
        };

        // Extract attachment links
        let mut attachments = Vec::new();
        let link_selector = Selector::parse("a[href]").unwrap();

        if let Some(content_div) = document.select(&content_selector).next() {
            for link in content_div.select(&link_selector) {
                if let Some(href) = link.value().attr("href") {
                    // Check if it's a file attachment (often contains "file" or "upload" in the URL)
                    if href.contains("/upload/file/") || href.contains("static/upload/file/") {
                        // Try to parse the absolute URL
                        if let Ok(absolute_url) = Url::parse(href) {
                            // Extract the filename from the link text or title attribute
                            let name = link
                                .value()
                                .attr("title")
                                .or_else(|| link.text().next())
                                .unwrap_or("Unnamed attachment")
                                .to_string();

                            attachments.push(Attachment {
                                name,
                                url: absolute_url,
                            });
                        } else {
                            // If it's a relative URL, combine it with base URL
                            let base_url = "https://jw.scut.edu.cn";
                            if let Ok(absolute_url) = Url::parse(&format!("{}{}", base_url, href)) {
                                // Extract the filename from the link text or title attribute
                                let name = link
                                    .value()
                                    .attr("title")
                                    .or_else(|| link.text().next())
                                    .unwrap_or("Unnamed attachment")
                                    .to_string();

                                attachments.push(Attachment {
                                    name,
                                    url: absolute_url,
                                });
                            }
                        }
                    }
                }
            }
        }

        NoticeDetail {
            metadata: notice_id,
            content,
            attachments,
        }
    }

    // Helper method to extract clean text content from HTML
    fn extract_clean_content(&self, content_div: &ElementRef) -> String {
        let mut clean_content = String::new();

        // Extract title
        if let Some(title_elem) = content_div
            .select(&Selector::parse("h3.content-title").unwrap())
            .next()
        {
            clean_content.push_str(&format!(
                "{}\n\n",
                title_elem.text().collect::<Vec<_>>().join("")
            ));
        }

        // Extract date
        if let Some(date_elem) = content_div
            .select(&Selector::parse("h5.content-date").unwrap())
            .next()
        {
            clean_content.push_str(&format!(
                "{}\n\n",
                date_elem.text().collect::<Vec<_>>().join("")
            ));
        }

        // Process paragraphs and other elements
        let paragraph_selector = Selector::parse("p").unwrap();
        let heading_selector = Selector::parse("h1, h2, h3, h4, h5, h6").unwrap();

        // First pass to extract text from regular paragraphs and headings
        for element in content_div.select(&paragraph_selector) {
            let text = element.text().collect::<Vec<_>>().join("");
            if !text.trim().is_empty() {
                clean_content.push_str(&format!("{}\n\n", text.trim()));
            }
        }

        for element in content_div.select(&heading_selector) {
            // Skip the title and date we already processed
            if element.value().id() == Some("content-title")
                || element.value().id() == Some("content-date")
            {
                continue;
            }

            let text = element.text().collect::<Vec<_>>().join("");
            if !text.trim().is_empty() {
                clean_content.push_str(&format!("{}\n\n", text.trim()));
            }
        }

        clean_content.trim().to_string()
    }
}
