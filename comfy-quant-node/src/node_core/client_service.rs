use anyhow::anyhow;
use bon::bon;
use comfy_quant_exchange::client::{
    spot_client::base::{SpotClientRequest, SpotClientResponse},
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

#[allow(async_fn_in_trait)]
pub trait SpotClientService {
    fn spot_client_svc(
        &self,
        client: &SpotClientKind,
        retry_max_retries: u64,
        retry_wait_secs: u64,
        timeout_secs: u64,
    ) -> BoxService<SpotClientRequest, SpotClientResponse, BoxError> {
        let svc = client.clone();
        let retry_policy = Attempts::builder()
            .max_retries(retry_max_retries)
            .wait_secs(retry_wait_secs)
            .build();
        let timeout_secs = Duration::from_secs(timeout_secs);

        ServiceBuilder::new()
            .retry(retry_policy)
            .timeout(timeout_secs)
            .service(svc)
            .boxed()
    }

    async fn spot_client_svc_call(
        &self,
        svc: &mut BoxService<SpotClientRequest, SpotClientResponse, BoxError>,
        req: SpotClientRequest,
    ) -> anyhow::Result<SpotClientResponse> {
        let res = svc
            .ready()
            .await
            .map_err(|e| anyhow!(e))?
            .call(req)
            .await
            .map_err(|e| anyhow!(e))?;

        Ok(res)
    }
}
