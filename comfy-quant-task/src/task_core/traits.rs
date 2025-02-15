use anyhow::Result;

#[allow(async_fn_in_trait)]
pub trait Executable {
    type Output;

    // 检查数据是否完整，如果完整则返回true，否则返回false
    // 如果数据不完整，则需要重新下载数据
    async fn check_data_complete(&self) -> Result<bool>;

    // 执行任务
    async fn execute(&self) -> Result<Self::Output>;
}
