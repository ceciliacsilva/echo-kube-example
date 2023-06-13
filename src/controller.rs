use kube::{
    runtime::{
        controller::Action,
        finalizer::{finalizer, Event as Finalizer},
    },
    Client,
};
use kube::{Api, ResourceExt};
use std::sync::Arc;
use std::time::Duration;
use tracing::warn;

use crate::error::Error;
use crate::spec::{self, HttpEcho};

/// From https://hub.docker.com/r/hashicorp/http-echo:
///      http-echo is an in-memory web server that renders an HTML page
///      containing the contents of the arguments provided to it.
///      This is especially useful for demos or a more extensive
///      "hello world" Docker application.
static POD_IMAGE: &str = "hashicorp/http-echo";
/// Default `hashcorp/http-echo` port.
static POD_PORT: i32 = 5678;
pub static ECHO_FINALIZER: &str = "echo.test.com";

/// Context for the `reconciler`.
#[derive(Clone)]
pub struct Context {
    /// kubernetes client.
    pub client: Client,
}

impl Context {
    /// Create a `new` `Context`.
    pub fn new(client: Client) -> Self {
        Context { client }
    }
}

/// Controller's `reconcile` logic.
pub(crate) async fn reconcile(
    http_echo: Arc<spec::HttpEcho>,
    ctx: Arc<Context>,
) -> Result<Action, Error> {
    // Should be impossible not have a namespace.
    let namespace = http_echo.namespace().unwrap();
    let echo_api: Api<spec::HttpEcho> = Api::namespaced(ctx.client.clone(), &namespace);
    finalizer(&echo_api, ECHO_FINALIZER, http_echo, |event| async {
        match event {
            // XXX: must be idempotent
            Finalizer::Apply(echo) => echo.reconcile(ctx.clone(), POD_IMAGE, POD_PORT).await,
            // XXX: must be idempotent
            Finalizer::Cleanup(echo) => echo.cleanup(ctx.clone()).await,
        }
    })
    .await
    .map_err(|e| {
        warn!("\nerro aqui:{:?}\n", e);
        Error::InputError("oi".to_owned())
    })
}

/// Controller::reconciler, `on_error` do this.
pub(crate) fn on_error(echo: Arc<HttpEcho>, error: &Error, _ctx: Arc<Context>) -> Action {
    warn!("Reconcile for {:?} failed: {:?}", echo, error);
    Action::requeue(Duration::from_secs(5 * 60))
}
