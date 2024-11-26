use anyhow::{anyhow, Result};
use bon::bon;
use comfy_quant_exchange::client::{
    spot_client::base::{
        AccountInformation, Balance, SpotClientRequest, SpotClientResponse, SymbolInformation,
    },
    spot_client_kind::SpotClientKind,
};
use futures::future;
use std::{thread::sleep, time::Duration};
use tower::{retry::Policy, util::BoxService, BoxError, Service, ServiceBuilder, ServiceExt};

#[derive(Clone)]
pub struct Attempts {
    max_retries: u64,
    wait_secs: Option<u64>,
}

#[bon]
impl Attempts {
    #[builder]
    fn new(max_retries: u64, wait_secs: Option<u64>) -> Self {
        Attempts {
            max_retries,
            wait_secs,
        }
    }
}

impl<Req, Res, E> Policy<Req, Res, E> for Attempts
where
    Req: Clone,
{
    type Future = future::Ready<()>;

    fn retry(&mut self, _req: &mut Req, result: &mut Result<Res, E>) -> Option<Self::Future> {
        match result {
            Ok(_) => None,
            Err(_) => {
                if self.max_retries > 0 {
                    self.max_retries -= 1;

                    if let Some(wait_secs) = self.wait_secs {
                        sleep(Duration::from_secs(wait_secs));
                    }

                    Some(future::ready(()))
                } else {
                    None
                }
            }
        }
    }

    fn clone_request(&mut self, req: &Req) -> Option<Req> {
        Some(req.clone())
    }
}

type SpotClientServiceInner = BoxService<SpotClientRequest, SpotClientResponse, BoxError>;

pub struct SpotClientService {
    inner: SpotClientServiceInner,
}

impl AsRef<SpotClientServiceInner> for SpotClientService {
    fn as_ref(&self) -> &SpotClientServiceInner {
        &self.inner
    }
}

impl AsMut<SpotClientServiceInner> for SpotClientService {
    fn as_mut(&mut self) -> &mut SpotClientServiceInner {
        &mut self.inner
    }
}

#[bon]
impl SpotClientService {
    #[builder]
    pub fn new(
        client: &SpotClientKind,
        retry_max_retries: u64,
        retry_wait_secs: u64,
        timeout_secs: u64,
    ) -> Self {
        let svc = client.clone();
        let retry_policy = Attempts::builder()
            .max_retries(retry_max_retries)
            .wait_secs(retry_wait_secs)
            .build();
        let timeout_secs = Duration::from_secs(timeout_secs);

        let inner = ServiceBuilder::new()
            .retry(retry_policy)
            .timeout(timeout_secs)
            .service(svc)
            .boxed();

        SpotClientService { inner }
    }

    pub async fn get_account(&mut self) -> Result<AccountInformation> {
        let req = SpotClientRequest::get_account();
        self.ready_call(req).await?.try_into()
    }

    pub async fn get_balance(&mut self, asset: &str) -> Result<Balance> {
        let req = SpotClientRequest::get_balance(asset);
        self.ready_call(req).await?.try_into()
    }

    pub async fn get_symbol_info(
        &mut self,
        base_asset: &str,
        quote_asset: &str,
    ) -> Result<SymbolInformation> {
        let req = SpotClientRequest::get_symbol_info(base_asset, quote_asset);
        self.ready_call(req).await?.try_into()
    }

    pub async fn platform_name(&mut self) -> Result<String> {
        let req = SpotClientRequest::platform_name();
        self.ready_call(req).await?.try_into()
    }

    async fn ready_call(&mut self, req: SpotClientRequest) -> Result<SpotClientResponse> {
        let res = self
            .as_mut()
            .ready()
            .await
            .map_err(|e| anyhow!(e))?
            .call(req)
            .await
            .map_err(|e| anyhow!(e))?;

        Ok(res)
    }
}
