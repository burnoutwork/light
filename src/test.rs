use crate::server::ProxyListener;

#[tokio::test]
async fn test_main() {
    let mut listener = ProxyListener::bind("127.0.0.1:8787").await.unwrap();
    listener.run().await.unwrap();
}