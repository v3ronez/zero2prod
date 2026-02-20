use zero2prod::run;

#[tokio::test]
async fn health_check() {
    spawn_app();
    let client = reqwest::Client::new();
    let response = client
        .get("http://localhost:8000/v1/health_check")
        .send()
        .await
        .expect("Failed to execute request ");
    assert_eq!(response.status(), 200);
}

fn spawn_app() -> () {
    let server = run().expect("Failed to bind address");
    let _ = tokio::spawn(server);
    ()
}
