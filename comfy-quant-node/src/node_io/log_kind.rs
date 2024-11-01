#[allow(unused)]
#[derive(Debug, Clone)]
pub(crate) enum LogKind {
    // 受益
    // 卖出
    // 错误
    // 买入
    // 撤销
    // 信息
}

#[allow(unused)]
pub(crate) enum SystemLog {
    Startup,      // 系统启动
    Shutdown,     // 系统关闭
    ConfigChange, // 配置变更
}

/// 交易
#[allow(unused)]
pub(crate) enum TradeLog {
    /// 买入
    OrderPlacement {
        order_id: String, // 订单ID
        platform: String, // 平台
        symbol: String,   // 币种
        market: String,   // 市场
        side: String,     // 方向
        price: f64,       // 价格
        amount: f64,      // 数量
    }, // 下单
    OrderCancellation {
        platform: String, // 平台
        order_id: String, // 订单ID
    }, // 撤销
    TradeExecution {
        platform: String, // 平台
        order_id: String, // 订单ID
    }, // 成交
    PositionUpdate {
        platform: String, // 平台
        before: f64,      // 更新前
        after: f64,       // 更新后
    }, // 仓位更新
}

#[allow(unused)]
pub enum AccountLog {
    FundChange,         // 资金变动
    BalanceUpdate,      // 余额更新
    LeverageAdjustment, // 杠杆调整
}

#[allow(unused)]
pub(crate) enum ErrorLog {
    /// 错误
    Error {
        platform: String, // 平台
        symbol: String,   // 币种
        market: String,   // 市场
        action: String,   // 操作
        message: String,  // 消息
    },
}
