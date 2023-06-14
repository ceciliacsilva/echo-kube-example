use futures::stream::StreamExt;
use kube::{api::ListParams, runtime::watcher::Config, runtime::Controller, Api, Client};
use std::sync::Arc;
use tracing::{error, info};

use crate::{controller::Context, crd::Echo};

mod controller;
mod crd;
mod deployment;
mod error;
mod finalizer;
mod service;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let client: Client = Client::try_default()
        .await
        .expect("Couldn't find a valid KUBECONFIG environment variable.");

    info!("Stating `http_echo` CustomResource.");

    let crd_api: Api<Echo> = Api::all(client.clone());
    let context: Arc<Context> = Arc::new(Context::new(client));

    if let Err(e) = crd_api.list(&ListParams::default().limit(1)).await {
        error!("CRD is not queryable; {e:?}. Is the CRD installed?");
        info!("Installation: kubectl apply -f echoes.test.com.yaml | cargo run");
        std::process::exit(1);
    }

    Controller::new(crd_api.clone(), Config::default())
        .owns(crd_api, Config::default())
        .shutdown_on_signal()
        .run(controller::reconcile, controller::on_error, context)
        .for_each(|reconciliation_result| async move {
            match reconciliation_result {
                Ok(echo_resource) => {
                    info!("Reconciliation successful. Resource: {:?}", echo_resource);
                }
                Err(reconciliation_err) => {
                    error!("Reconciliation error: {:?}", reconciliation_err)
                }
            }
        })
        .await
}
