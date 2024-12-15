use super::BinanceClient;

#[allow(unused)]
#[derive(Clone)]
pub struct SpotWebsocket<'a> {
    client: &'a BinanceClient,
}

impl<'a> SpotWebsocket<'a> {
    pub fn new(client: &'a BinanceClient) -> Self {
        SpotWebsocket { client }
    }
}
