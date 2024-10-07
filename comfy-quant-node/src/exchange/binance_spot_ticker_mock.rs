use crate::{
    base::{
        traits::node::{NodeExecutor, NodePorts},
        Ports, Slot,
    },
    data::{ExchangeInfo, Ticker},
    workflow,
};
use anyhow::Result;
use bon::Builder;
use chrono::{DateTime, Utc};
use comfy_quant_api::task::executor::run_binance_klines_task;

#[derive(Builder, Debug, Clone)]
#[builder(on(String, into))]
pub struct Widget {
    base_currency: String,
    quote_currency: String,
    start_datetime: DateTime<Utc>,
    end_datetime: DateTime<Utc>,
}

#[allow(unused)]
pub struct BinanceSpotTickerMock {
    pub(crate) widget: Widget,
    pub(crate) ports: Ports,
}

impl BinanceSpotTickerMock {
    pub fn try_new(widget: Widget) -> Result<Self> {
        let mut ports = Ports::new();

        let exchange_info = ExchangeInfo::builder()
            .name("binance")
            .market("spot")
            .base_currency(&widget.base_currency)
            .quote_currency(&widget.quote_currency)
            .build();

        ports.add_output(
            0,
            Slot::<ExchangeInfo>::builder().data(exchange_info).build(),
        )?;

        ports.add_output(1, Slot::<Ticker>::builder().channel_capacity(1024).build())?;

        Ok(BinanceSpotTickerMock { widget, ports })
    }

    async fn output1(&self) -> Result<()> {
        let slot1 = self.ports.get_output::<Ticker>(1)?;
        let symbol = format!(
            "{}{}",
            self.widget.base_currency, self.widget.quote_currency
        )
        .to_uppercase();
        let start_timestamp = self.widget.start_datetime.timestamp();
        let end_timestamp = self.widget.end_datetime.timestamp();

        let receiver =
            run_binance_klines_task("spot", symbol, "1s", start_timestamp, end_timestamp).await?;

        // todo: 从数据库推送中获取行情
        tokio::spawn(async move {
            loop {
                let ticker = Ticker::builder()
                    .timestamp(Utc::now().timestamp())
                    .price(0.)
                    .build();

                slot1.send(ticker)?;

                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }

            #[allow(unreachable_code)]
            Ok::<(), anyhow::Error>(())
        });

        Ok(())
    }
}

impl NodePorts for BinanceSpotTickerMock {
    fn get_ports(&self) -> Result<&Ports> {
        Ok(&self.ports)
    }

    fn get_ports_mut(&mut self) -> Result<&mut Ports> {
        Ok(&mut self.ports)
    }
}

impl NodeExecutor for BinanceSpotTickerMock {
    async fn execute(&mut self) -> Result<()> {
        self.output1().await?;
        Ok(())
    }
}

impl TryFrom<workflow::Node> for BinanceSpotTickerMock {
    type Error = anyhow::Error;

    fn try_from(node: workflow::Node) -> Result<Self> {
        if node.properties.prop_type != "data.BinanceSpotTickerMock" {
            anyhow::bail!(
                "Try from workflow::Node to BinanceSpotTickerMock failed: Invalid prop_type"
            );
        }

        let [base_currency, quote_currency, start_datetime, end_datetime] =
            node.properties.params.as_slice()
        else {
            anyhow::bail!(
                "Try from workflow::Node to BinanceSpotTickerMock failed: Invalid params"
            );
        };

        let base_currency = base_currency.as_str().ok_or(anyhow::anyhow!(
            "Try from workflow::Node to BinanceSpotTickerMock failed: Invalid base_currency"
        ))?;

        let quote_currency = quote_currency.as_str().ok_or(anyhow::anyhow!(
            "Try from workflow::Node to BinanceSpotTickerMock failed: Invalid quote_currency"
        ))?;

        let start_datetime = start_datetime
            .as_str()
            .ok_or(anyhow::anyhow!(
                "Try from workflow::Node to BinanceSpotTickerMock failed: Invalid start_datetime"
            ))?
            .parse::<DateTime<Utc>>()?;

        let end_datetime = end_datetime
            .as_str()
            .ok_or(anyhow::anyhow!(
                "Try from workflow::Node to BinanceSpotTickerMock failed: Invalid end_datetime"
            ))?
            .parse::<DateTime<Utc>>()?;

        let widget = Widget::builder()
            .base_currency(base_currency)
            .quote_currency(quote_currency)
            .start_datetime(start_datetime)
            .end_datetime(end_datetime)
            .build();

        BinanceSpotTickerMock::try_new(widget)
    }
}
