use futures::StreamExt;
use kube::{api::ListParams, runtime::watcher::Config, runtime::Controller, Api, Client};
use std::sync::Arc;
use tracing::{error, info};
use tracing_subscriber;

use crate::{controller::Context, spec::HttpEcho};

mod controller;
mod deployment;
mod error;
mod finalizer;
mod service;
mod spec;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let client: Client = Client::try_default()
        .await
        .expect("Couldn't find a valid KUBECONFIG environment variable.");

    info!("Stating `http_echo` CustomResource.");

    let crd_api: Api<HttpEcho> = Api::all(client.clone());
    let context: Arc<Context> = Arc::new(Context::new(client));

    if let Err(e) = crd_api.list(&ListParams::default().limit(1)).await {
        error!("CRD is not queryable; {e:?}. Is the CRD installed?");
        info!("Installation: cargo run --bin crdgen | kubectl apply -f -");
        std::process::exit(1);
    }

    // TODO: missing `OwnerReferences`.
    Controller::new(crd_api.clone(), Config::default())
        .run(controller::reconcile, controller::on_error, context)
        .for_each(|reconciliation_result| async move {
            match reconciliation_result {
                Ok(echo_resource) => {
                    println!("Reconciliation successful. Resource: {:?}", echo_resource);
                }
                Err(reconciliation_err) => {
                    eprintln!("Reconciliation error: {:?}", reconciliation_err)
                }
            }
        })
        //.filter_map(|x| async move { std::result::Result::ok(x) })
        //.for_each(|_| futures::future::ready(()))
        .await
}
