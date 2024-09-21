use serde_json::Value;
use futures_util::{future, pin_mut, StreamExt, SinkExt};
use serde_json::{json};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

use crate::radio_alert::play_sound;


#[derive(Debug)]
pub struct MempoolBlockInfo {
    /// 预计区块
    pub index: usize,
    /// 中位手续费
    pub median_fee: f64,
    /// 费用范围
    pub fee_range: Vec<f64>,
    /// 总手续费
    pub total_fees: f64,
    /// 交易数量
    pub n_tx: u64,
}


pub async fn monitor_mempool_blocks_fee(alert_fee: f64,moniter_block_num: usize) -> Result<(), Box<dyn std::error::Error>> {
    println!("### 开始监控手续费 ###");
    println!("### 当低于{}聪/字节时报警 ###", alert_fee);
    println!("### 监控前{}个区块 ###", moniter_block_num);
    let (ws_stream, _) = connect_async("wss://mempool.fractalbitcoin.io/api/v1/ws").await?;
    println!("WebSocket 连接已成功建立");

    let (mut write, read) = ws_stream.split();

    let init_message = json!({"action": "init"});
    write.send(Message::Text(init_message.to_string())).await?;

    let want_message = json!({"action": "want", "data": ["mempool-blocks"]});
    write.send(Message::Text(want_message.to_string())).await?;

    let handle_messages = read.for_each(|message| async {
        match message {
            Ok(Message::Text(text)) => {
                if let Ok(data) = serde_json::from_str::<Value>(&text) {
                    let result = process_mempool_blocks(&data);
                    // 监控前 moniter_block_num 个区块
                    let moniter_blocks = result.iter().take(moniter_block_num).collect::<Vec<&MempoolBlockInfo>>();
                    // 当最低手续费小于alert_fee时报警
                    if moniter_blocks.iter().any(|block| block.fee_range[0] < alert_fee) {
                        let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
                        for block in moniter_blocks.iter().filter(|b| b.fee_range[0] < alert_fee) {
                            println!("[{}] 预计区块: +{}, 中位手续费: ~{:.2} 聪/字节, 费用范围: {:.2} - {:.2} 聪/字节, 总手续费: {:.2} FB, 交易数量: {}",
                                now,
                                block.index,
                                block.median_fee,
                                block.fee_range[0],
                                block.fee_range[1],
                                block.total_fees,
                                block.n_tx
                            );
                        }
                        // 播放报警声音
                        play_sound();
                    }
                }
            }
            Ok(Message::Close(_)) => {
                println!("### 连接关闭 ###");
            }
            Err(e) => {
                eprintln!("错误: {}", e);
            }
            _ => {}
        }
    });

    pin_mut!(handle_messages);
    handle_messages.await;
    Ok(())

}



fn process_mempool_blocks(data: &Value) -> Vec<MempoolBlockInfo> {
    let mut result = vec![];
    if let Some(mempool_blocks) = data.get("mempool-blocks") {
        if let Some(blocks) = mempool_blocks.as_array() {
            for (i, block) in blocks.iter().enumerate() {
                if let (Some(median_fee), Some(fee_range), Some(total_fees), Some(n_tx)) = (
                    block.get("medianFee"),
                    block.get("feeRange"),
                    block.get("totalFees"),
                    block.get("nTx"),
                ) {
                    let min_fee = fee_range.as_array().and_then(|v| v.first()).and_then(|v| v.as_f64()).unwrap_or(0.0);
                    let max_fee = fee_range.as_array().and_then(|v| v.last()).and_then(|v| v.as_f64()).unwrap_or(0.0);
                    let total_fees = total_fees.as_f64().unwrap_or(0.0) / 100_000_000.0;
                    
                    result.push(MempoolBlockInfo {
                        index: i,
                        median_fee: median_fee.as_f64().unwrap_or(0.0),
                        fee_range: vec![min_fee, max_fee],
                        total_fees: total_fees,
                        n_tx: n_tx.as_u64().unwrap_or(0),
                    });
                }
            }
        }
    }
    result
}

