use comfy_quant_api::setting::Setting;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let setting = Setting::try_new()?;

    println!("setting: {:?}", setting);
    Ok(())
}
