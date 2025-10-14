use crate::helpers::{drop_database, spawn_app};

#[tokio::test]
async fn health_check_works() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/health", &app.address))
        .send()
        .await
        .expect("Failed to execute request.");
    println!("\ntest server run on address: {}\n", &app.address);

    assert!(response.status().is_success());
    assert_eq!(response.content_length(), Some(0));

    //reset db
    drop_database(&app.connection_pool).await;
}
