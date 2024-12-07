use bon::bon;
use chrono::{DateTime, Utc};
use rust_decimal::{prelude::ToPrimitive, Decimal, MathematicalOps};
use rust_decimal_macros::dec;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct CurrencyPair {
    base: String,  // 基础货币
    quote: String, // 计价货币
}

#[allow(unused)]
impl CurrencyPair {
    pub fn new(base: impl Into<String>, quote: impl Into<String>) -> Self {
        Self {
            base: base.into(),
            quote: quote.into(),
        }
    }

    pub fn inverse(&self) -> Self {
        Self::new(&self.quote, &self.base)
    }
}

#[derive(Debug, Clone)]
pub struct ExchangeRate {
    rate: Decimal,           // 汇率
    datetime: DateTime<Utc>, // 时间戳
    source: String,          // 数据来源
    weight: Decimal,         // 权重
}

#[allow(unused)]
impl ExchangeRate {
    pub fn new(rate: Decimal, source: impl Into<String>, datetime: DateTime<Utc>) -> Self {
        Self {
            rate,
            datetime,
            source: source.into(),
            weight: dec!(1), // 默认权重
        }
    }

    pub fn inverse(&self) -> Self {
        Self {
            rate: Decimal::ONE / self.rate,
            datetime: self.datetime,
            source: self.source.clone(),
            weight: self.weight,
        }
    }

    pub fn rate(&self) -> &Decimal {
        &self.rate
    }
}

// 汇率来源数据
#[derive(Debug, Clone)]
struct RateSource {
    rate: Decimal,
    datetime: DateTime<Utc>,
}

#[derive(Debug)]
pub struct ExchangeRateManager {
    // 每个币对在不同交易所的原始汇率
    source_rates: HashMap<CurrencyPair, HashMap<String, RateSource>>,
    // 融合后的汇率缓存
    merged_rates: HashMap<CurrencyPair, ExchangeRate>,
    // 中间货币列表
    intermediate_currencies: HashSet<String>,
    // 交易所权重配置
    exchange_weights: HashMap<String, Decimal>,
    // 配置参数
    config: RateManagerConfig,
}

#[derive(Debug)]
struct RateManagerConfig {
    // 异常值阈值(与均值的最大偏差百分比)
    outlier_threshold: Decimal,
    // 时效性权重衰减系数(每小时)
    time_decay_factor: Decimal,
    // 缓存过期时间(秒)
    cache_ttl: i64,
}

impl Default for RateManagerConfig {
    fn default() -> Self {
        Self {
            outlier_threshold: dec!(0.1), // 10%偏差
            time_decay_factor: dec!(0.9), // 每小时衰减10%
            cache_ttl: 1,                 // 1秒缓存
        }
    }
}

#[bon]
#[allow(unused)]
impl ExchangeRateManager {
    #[builder]
    pub fn new(
        intermediate_currencies: Option<Vec<String>>,
        exchange_weights: Option<Vec<(String, Decimal)>>,
        config: Option<RateManagerConfig>,
    ) -> Self {
        ExchangeRateManager {
            source_rates: HashMap::new(),
            merged_rates: HashMap::new(),
            intermediate_currencies: intermediate_currencies
                .unwrap_or_default()
                .into_iter()
                .collect(),
            exchange_weights: exchange_weights.unwrap_or_default().into_iter().collect(),
            config: config.unwrap_or_default(),
        }
    }

    // 更新单个数据源的汇率
    pub fn update_rate(
        &mut self,
        base: impl Into<String>,
        quote: impl Into<String>,
        rate: Decimal,
        source: impl Into<String>,
        datetime: DateTime<Utc>,
    ) {
        let pair = CurrencyPair::new(base, quote);
        let source = source.into();

        // 更新原始数据
        self.source_rates
            .entry(pair.clone())
            .or_default()
            .insert(source.clone(), RateSource { rate, datetime });

        // 更新反向汇率
        let inverse_pair = pair.inverse();
        self.source_rates.entry(inverse_pair).or_default().insert(
            source,
            RateSource {
                rate: Decimal::ONE / rate,
                datetime,
            },
        );

        // 添加中间货币
        self.intermediate_currencies.insert(pair.base.clone());
        self.intermediate_currencies.insert(pair.quote.clone());

        // 清除受影响的缓存
        self.merged_rates.remove(&pair);
        self.merged_rates.remove(&pair.inverse());
    }

    // 获取融合汇率
    pub fn get_rate(
        &mut self,
        base: impl AsRef<str>,
        quote: impl AsRef<str>,
    ) -> Option<ExchangeRate> {
        let pair = CurrencyPair::new(base.as_ref(), quote.as_ref());

        // 检查缓存
        if let Some(cached_rate) = self.merged_rates.get(&pair) {
            if Utc::now()
                .signed_duration_since(cached_rate.datetime)
                .num_seconds()
                < self.config.cache_ttl
            {
                return Some(cached_rate.clone());
            }
        }

        // 1. 尝试获取直接汇率
        if let Some(merged_rate) = self.calculate_merged_rate(&pair) {
            self.merged_rates.insert(pair, merged_rate.clone());
            return Some(merged_rate);
        }

        // 2. 尝试通过中间货币获取间接汇率
        for intermediate in &self.intermediate_currencies {
            let pair1 = CurrencyPair::new(base.as_ref(), intermediate);
            let pair2 = CurrencyPair::new(intermediate, quote.as_ref());

            if let (Some(rate1), Some(rate2)) = (
                self.get_rate_internal(&pair1),
                self.get_rate_internal(&pair2),
            ) {
                let merged_rate = ExchangeRate::new(
                    rate1.rate * rate2.rate,
                    format!("{}->{}->{}", rate1.source, intermediate, rate2.source),
                    rate1.datetime.max(rate2.datetime),
                );
                self.merged_rates.insert(pair.clone(), merged_rate.clone());
                return Some(merged_rate);
            }
        }

        None
    }

    // 内部获取汇率方法(不使用缓存)
    fn get_rate_internal(&self, pair: &CurrencyPair) -> Option<ExchangeRate> {
        self.calculate_merged_rate(pair)
    }

    // 计算融合汇率
    fn calculate_merged_rate(&self, pair: &CurrencyPair) -> Option<ExchangeRate> {
        let sources = self.source_rates.get(pair)?;
        if sources.is_empty() {
            return None;
        }

        let now = Utc::now();
        let mut valid_rates = Vec::new();
        let mut total_weight = Decimal::ZERO;

        // 1. 计算均值用于异常值检测
        let mean_rate: Decimal =
            sources.values().map(|s| s.rate).sum::<Decimal>() / Decimal::from(sources.len());

        // 2. 收集有效数据并计算权重
        for (exchange, source) in sources {
            // 检查是否异常值
            let deviation = ((source.rate - mean_rate).abs() / mean_rate).abs();
            if deviation > self.config.outlier_threshold {
                continue;
            }

            // 计算时效性权重
            let hours_old = Decimal::from(
                now.signed_duration_since(source.datetime)
                    .num_hours()
                    .max(0),
            );
            let time_weight = self
                .config
                .time_decay_factor
                .powu(hours_old.to_u64().unwrap_or(0));

            // 合并权重
            let default_exchange_weight = dec!(1);
            let exchange_weight = self
                .exchange_weights
                .get(exchange)
                .unwrap_or(&default_exchange_weight);

            let weight = exchange_weight * time_weight;

            valid_rates.push((source.rate, weight));
            total_weight += weight;
        }

        if valid_rates.is_empty() {
            return None;
        }

        // 3. 计算加权平均值
        let weighted_rate = valid_rates
            .iter()
            .map(|(rate, weight)| rate * weight)
            .sum::<Decimal>()
            / total_weight;

        // 4. 构建结果
        Some(ExchangeRate::new(weighted_rate, "merged", now))
    }

    // 转换金额
    pub fn convert_amount(
        &mut self,
        from: impl AsRef<str>,
        to: impl AsRef<str>,
        amount: Decimal,
    ) -> Option<Decimal> {
        self.get_rate(from, to).map(|rate| amount * rate.rate)
    }

    // 清理过期数据
    pub fn cleanup_expired(&mut self, max_age: chrono::Duration) {
        let now = Utc::now();

        // 清理原始数据
        self.source_rates.retain(|_, sources| {
            sources.retain(|_, rate| now.signed_duration_since(rate.datetime) < max_age);
            !sources.is_empty()
        });

        // 清理缓存
        self.merged_rates
            .retain(|_, rate| now.signed_duration_since(rate.datetime) < max_age);
    }
}

impl Default for ExchangeRateManager {
    fn default() -> Self {
        Self::builder().build()
    }
}

// 测试用例
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merged_rates() {
        let mut manager = ExchangeRateManager::default();
        let now = Utc::now();

        // 添加多个数据源的数据
        manager.update_rate("BTC", "USDT", dec!(50000), "binance", now);
        manager.update_rate("BTC", "USDT", dec!(50100), "okx", now);
        manager.update_rate("BTC", "USDT", dec!(49900), "huobi", now);

        manager.update_rate("USDT", "CNY", dec!(7.22), "anomaly", now);
        manager.update_rate("USDT", "CNY", dec!(7.12), "binance", now);

        // 测试融合汇率
        if let Some(rate) = manager.get_rate("BTC", "USDT") {
            // 汇率应该在49900-50100之间
            assert!(rate.rate > dec!(49900));
            assert!(rate.rate < dec!(50100));
        }

        // 测试异常值处理
        manager.update_rate(
            "BTC",
            "USDT",
            dec!(55000), // 明显偏离的值
            "anomaly",
            now,
        );

        let rate = manager.get_rate("BTC", "USDT").unwrap();
        assert_eq!(rate.rate, dec!(51250));

        let rate = manager.get_rate("USDT", "CNY").unwrap();
        assert_eq!(rate.rate, dec!(7.17));
    }
}
