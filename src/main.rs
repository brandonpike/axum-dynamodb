use axum::{routing, Router};
use tokio::net::TcpListener;

#[tokio::main(worker_threads = 1)]
async fn main() -> anyhow::Result<()> {
    let addr = "127.0.0.1:8080";

    let listener = TcpListener::bind(addr).await?;
    let server_handle = tokio::spawn(run_server(listener));

    // Test client that routes to a dynamodb get function
    run_test_client(addr).await?;

    // Runs until server exits.
    server_handle.await??;

    Ok(())
}

async fn run_test_client(addr: &str) -> anyhow::Result<()> {
    let body = reqwest::get(format!("http://{addr}/get_item"))
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    println!("body = {body:?}");

    Ok(())
}

async fn run_server(listener: TcpListener) -> anyhow::Result<()> {
    let router = Router::new().route("/get_item", routing::get(routes::get_item));

    axum::serve(listener, router)
        .await
        .map_err(|err| err.into())
}

mod routes {
    use aws_sdk_dynamodb::types::AttributeValue;

    pub async fn get_item() -> String {
        let config = aws_config::load_from_env().await;
        let client = aws_sdk_dynamodb::Client::new(&config);

        let note_id = AttributeValue::S("123".into());

        let maybe_item = client
            .get_item()
            .table_name("bpike-test")
            .key("noteid", note_id)
            .consistent_read(true)
            .send()
            .await
            .unwrap()
            .item;

        let map = maybe_item.unwrap();

        format!("{map:?}")
    }
}
