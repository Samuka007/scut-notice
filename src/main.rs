mod detail;
mod metadata;
mod util;

#[tokio::main]
async fn main() {
    // Set up logging
    util::setup_logging();

    let client = util::Client::new().await;

    let notices = client.test_fetch_notices().await;
    println!("Response: {:?}", notices);
    let detail = client
        .fetch_notice_detail(notices.list.as_ref().unwrap()[8].clone())
        .await;
    println!("Detail: {:?}", detail);
}
