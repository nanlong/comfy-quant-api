use async_lock::RwLock;
use serde::de::Deserializer;
use serde::ser::Serializer;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub fn serialize<S, T>(val: &[Arc<RwLock<T>>], s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    T: Serialize + Clone,
{
    // 创建临时 Vec 存储解锁后的值
    let temp: Vec<T> = val
        .iter()
        .map(|lock| (*lock.read_blocking()).clone())
        .collect();

    // 序列化临时 Vec
    temp.serialize(s)
}

pub fn deserialize<'de, D, T>(d: D) -> Result<Vec<Arc<RwLock<T>>>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    // 先反序列化成普通 Vec
    let temp = Vec::<T>::deserialize(d)?;

    // 将每个值包装成 Arc<RwLock<T>>
    let result = temp
        .into_iter()
        .map(|val| Arc::new(RwLock::new(val)))
        .collect();

    Ok(result)
}
