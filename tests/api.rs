pub mod support;

use support::app::TestApp;
use support::assert_is_redirected_to;

#[tokio::test]
async fn unauthorized_access_redirects_to_login() {
    let app = TestApp::new().await;
    let resp = app
        .api_client
        .get(format!("http://{}/hosts", &app.addr))
        .send()
        .await
        .unwrap();
    assert_is_redirected_to(&resp, "/login")
}

#[tokio::test]
async fn successful_login_redirects_to_hosts() {
    let app = TestApp::new().await;
    let resp = app.login().await;
    assert_is_redirected_to(&resp, "/hosts")
}

#[tokio::test]
async fn index_displays_available_hosts() {
    let mut app = TestApp::new().await;
    let host1 = app.gen.generate_host().await;
    let host2 = app.gen.generate_host().await;
    app.login().await;

    let resp = app
        .api_client
        .get(format!("http://{}/hosts", &app.addr))
        .send()
        .await
        .unwrap();
    let resp_html = resp.text().await.unwrap();

    assert!(resp_html.contains(&host1.ip.to_string()));
    assert!(resp_html.contains(&host2.ip.to_string()));
}
