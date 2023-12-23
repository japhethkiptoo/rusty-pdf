mod pdf;

use futures::future::ok;
use node_bindgen::derive::node_bindgen;
use serde::Deserialize;

use pdf::util::Transaction;
use tslink::tslink;

use crate::pdf::util::create_pdf;

#[tslink]
#[derive(Debug, Deserialize)]
struct Payload {
    pdf_name: String,
    transactions: Vec<Transaction>,
}

#[tslink]
#[node_bindgen]
async fn generate_statement(payload: String, mmf: bool) {
    let data: Payload = serde_json::from_str(payload.as_str()).expect("Data Error");

    let _result = async move {
        create_pdf(data.transactions, data.pdf_name, mmf);
        ok::<(), ()>(())
    }
    .await;
}
