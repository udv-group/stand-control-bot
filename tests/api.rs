pub mod support;

use support::app::TestApp;

#[tokio::test]
async fn index_displays_available_hosts() {
    let mut app = TestApp::new().await;
    let host1 = app.gen.generate_host().await;
    let host2 = app.gen.generate_host().await;
    let resp = app
        .api_client
        .get(format!("http://{}", &app.addr))
        .send()
        .await
        .unwrap();
    let resp_html = resp.text().await.unwrap();

    assert!(resp_html.contains(&host1.ip.to_string()));
    assert!(resp_html.contains(&host2.ip.to_string()));
}
