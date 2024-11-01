## 重要

1. 运行 Jaeger UI

   ```
    docker run -d -p4317:4317 -p16686:16686 jaegertracing/all-in-one:latest
   ```

---

1. 关键性能指标（总收益率、年化收益率、最大回撤等）
1. 每笔交易的详细信息
1. 价格走势和交易点可视化
1. 累计收益曲线
1. 回撤可视化
1. 交易收益分布图
1. 滚动夏普比率
1. 月度/年度收益表
1. 资金曲线
1. 波动率分析
1. 相关性分析（如果交易多个资产）

回测日志

    - 进度 0% - 100%
    - 耗时(秒/毫秒/)
    - 日志总数
    - 交易次数
    - 状态（运行中、已完成）

初始持仓信息 initil_positions

    - workflow_id
    - platform
    - market
    - asset
    - balance
    - created_at

持仓信息 positions

    - workflow_id
    - platform
    - market
    - asset
    - locked
    - free
    - total
    - avg_price
    - current_price
    - unrealized_pnl
    - unrealized_pnl_ratio
    - updated_at

交易记录 trades

    - workflow_id
    - platform_name
    - market
    - order_id
    - symbol
    - direction
    - price
    - quantity
    - commission
    - timestamp

日志信息 workflow_logs

    - workflow_id
    - platform
    - log_type
    - message
    - timestamp

docker run -d -p6831:6831/udp -p6832:6832/udp -p16686:16686 -p14268:14268 jaegertracing/all-in-one:latest

docker run -d -p4317:4317 -p16686:16686 jaegertracing/all-in-one:latest

docker run -d -p4317:4317 -p55679:55679 otel/opentelemetry-collector:latest

docker run -d -p4317:4317 -p55679:55679 otel/opentelemetry-collector-contrib:latest

docker run \
 -p 127.0.0.1:4317:4317 \
 -p 127.0.0.1:55679:55679 \
 otel/opentelemetry-collector-contrib:0.112.0 \
 2>&1 | tee collector-output.txt # Optionally tee output for easier search later
