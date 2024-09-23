// use crate::{
//     traits::Node,
//     types::{Account, Backtest, Input, Ticker},
//     utils::{get_input_clone, get_input_data},
// };
// use anyhow::Result;
// use futures::StreamExt;

// pub struct Params {
//     pub grid_size: f64,
//     pub grid_count: u32,
// }

// pub struct Inputs {
//     pub slot0: Option<Input<Ticker>>,
//     pub slot1: Option<Input<Account>>,
//     pub slot2: Option<Input<Backtest>>,
// }

// pub struct SpotGrid {
//     pub params: Params,
//     pub inputs: Inputs,
// }

// impl Node<Params, Inputs> for SpotGrid {
//     type Output = ();

//     fn new(params: Params, inputs: Inputs) -> Self {
//         Self { inputs, params }
//     }

//     async fn execute(&self) -> Result<Self::Output> {
//         let account = get_input_data(self.inputs.slot1.as_ref()).await?;
//         println!("account: {:?}", account);

//         let ticker_stream = get_input_clone(self.inputs.slot0.as_ref()).await?;

//         tokio::spawn(async move {
//             let mut ticker_stream = ticker_stream.lock().await;

//             while let Some(ticker) = ticker_stream.next().await {
//                 println!("ticker: {:?}", ticker);
//             }
//         });

//         Ok(())
//     }
// }
