use comfy_quant_api::helper::init_tracing_subscriber;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let server_name = "comfy-quant-api".to_string();
    let _guard = init_tracing_subscriber(server_name)?;

    foo().await;

    tokio::time::sleep(std::time::Duration::from_secs(10)).await;

    Ok(())
}

#[tracing::instrument]
async fn foo() {
    tracing::info!(
        monotonic_counter.foo = 1_u64,
        key_1 = "bar",
        key_2 = 10,
        "handle foo",
    );

    tracing::info!(histogram.baz = 10, "histogram example",);
}
