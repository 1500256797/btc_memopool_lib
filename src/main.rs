use btc_memopool_lib::api::fb_mempool::monitor_mempool_blocks_fee;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 低于xxxx 时进行报警
    // moniter block number [1,2,3]
    monitor_mempool_blocks_fee(560.0, 2).await?;
    Ok(())
}
