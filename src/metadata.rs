use serde::Deserialize;

use crate::util::Client;

// Define structures to parse the API response
#[derive(Debug, Deserialize)]
pub struct FindInformNotice {
    pub list: Option<Vec<NoticeMetadata>>,
    pub total: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct NoticeMetadata {
    pub id: String,
    pub title: String,
    pub createTime: String,
    pub label: Option<String>,
    pub isLastest: Option<bool>,
}

impl Client {
    pub async fn test_fetch_notices(&self) -> FindInformNotice {
        let api_url = "https://jw.scut.edu.cn/zhinan/cms/article/v2/findInformNotice.do";
        println!("Fetching data from {}", api_url);

        // Send a POST request to the API endpoint (simulating the AJAX call from the webpage)
        let response = self
            .client
            .post(api_url)
            .form(&[
                ("category", "0"),  // Category value from the HTML
                ("tag", "0"),       // Default to "all" notices
                ("pageNum", "1"),   // First page
                ("pageSize", "15"), // 15 items per page
                ("keyword", ""),    // No search keyword
            ])
            .send()
            .await
            .unwrap();

        // Check if the request was successful
        if !response.status().is_success() {
            panic!("Failed to fetch data: HTTP {}", response.status());
        }

        let body = response.text().await.unwrap();

        log::trace!("Response: {:?}", body);

        serde_json::from_str(&body).unwrap()
    }

    pub async fn fetch_notices_after_date(&self, date: chrono::NaiveDate) -> Vec<NoticeMetadata> {
        let mut all_notices = Vec::new();
        let mut page_num = 1;
        let page_size = 15;

        loop {
            let api_url = "https://jw.scut.edu.cn/zhinan/cms/article/v2/findInformNotice.do";
            println!("Fetching data from {}", api_url);

            // Send a POST request to the API endpoint (simulating the AJAX call from the webpage)
            let response = self
                .client
                .post(api_url)
                .form(&[
                    ("category", "0"),                    // Category value from the HTML
                    ("tag", "0"),                         // Default to "all" notices
                    ("pageNum", &page_num.to_string()),   // Current page number
                    ("pageSize", &page_size.to_string()), // Number of items per page
                    ("keyword", ""),                      // No search keyword
                ])
                .send()
                .await
                .unwrap();

            // Check if the request was successful
            if !response.status().is_success() {
                panic!("Failed to fetch data: HTTP {}", response.status());
            }

            let body = response.text().await.unwrap();

            log::trace!("Response: {:?}", body);
            let result: FindInformNotice = serde_json::from_str(&body).unwrap();
            if let Some(notices) = result.list {
                for notice in notices {
                    // Parse date in format "YYYY.MM.DD" more robustly
                    let notice_date = notice.createTime.split('.').collect::<Vec<&str>>();

                    // Default to a very old date if parsing fails, so it won't be included
                    let notice_time = if notice_date.len() == 3 {
                        match (
                            notice_date[0].parse::<i32>(),
                            notice_date[1].parse::<u32>(),
                            notice_date[2].parse::<u32>(),
                        ) {
                            (Ok(year), Ok(month), Ok(day)) => chrono::NaiveDate::from_ymd_opt(
                                year, month, day,
                            )
                            .unwrap_or_else(|| {
                                log::warn!("Invalid date components: {}-{}-{}", year, month, day);
                                chrono::NaiveDate::from_ymd_opt(1900, 1, 1).unwrap()
                            }),
                            _ => {
                                log::warn!(
                                    "Failed to parse date parts from: {}",
                                    notice.createTime
                                );
                                chrono::NaiveDate::from_ymd_opt(1900, 1, 1).unwrap()
                            }
                        }
                    } else {
                        log::warn!("Unexpected date format: {}", notice.createTime);
                        chrono::NaiveDate::from_ymd_opt(1900, 1, 1).unwrap()
                    };

                    if notice_time > date {
                        all_notices.push(notice);
                    } else {
                        // If the notice is older than the specified date, break the loop
                        break;
                    }
                }
            }
            if result.total <= (page_num * page_size) as u32 {
                break;
            }
            page_num += 1;
            // Sleep for a short duration to avoid overwhelming the server
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
        all_notices
    }

    pub async fn fetch_all_notices(&self) -> Vec<NoticeMetadata> {
        let mut all_notices = Vec::new();
        let mut page_num = 1;
        let page_size = 15;

        loop {
            let api_url = "https://jw.scut.edu.cn/zhinan/cms/article/v2/findInformNotice.do";
            println!("Fetching data from {}", api_url);

            // Send a POST request to the API endpoint (simulating the AJAX call from the webpage)
            let response = self
                .client
                .post(api_url)
                .form(&[
                    ("category", "0"),                    // Category value from the HTML
                    ("tag", "0"),                         // Default to "all" notices
                    ("pageNum", &page_num.to_string()),   // Current page number
                    ("pageSize", &page_size.to_string()), // Number of items per page
                    ("keyword", ""),                      // No search keyword
                ])
                .send()
                .await
                .unwrap();

            // Check if the request was successful
            if !response.status().is_success() {
                panic!("Failed to fetch data: HTTP {}", response.status());
            }

            let body = response.text().await.unwrap();

            log::trace!("Response: {:?}", body);
            let result: FindInformNotice = serde_json::from_str(&body).unwrap();
            if let Some(notices) = result.list {
                all_notices.extend(notices);
            }
            if result.total <= (page_num * page_size) as u32 {
                break;
            }
            page_num += 1;

            // Sleep for a short duration to avoid overwhelming the server
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
        all_notices
    }
}
