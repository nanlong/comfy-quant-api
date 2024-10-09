use bon::Builder;
use serde_json::Value;

#[derive(Builder, Debug, Clone)]
#[builder(on(String, into))]
#[allow(unused)]
pub struct ClientInformation {
    pub(crate) client_type: String,
    pub(crate) data: Option<Value>,
}
