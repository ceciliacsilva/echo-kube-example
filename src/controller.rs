use kube::Api;
use kube::{
    runtime::{
        controller::Action,
        finalizer::{finalizer, Event as Finalizer},
    },
    Client, ResourceExt,
};
use std::sync::Arc;
use std::time::Duration;

use crate::crd::{self, Echo};
use crate::error::Error;

/// From https://hub.docker.com/r/hashicorp/http-echo:
///      http-echo is an in-memory web server that renders an HTML page
///      containing the contents of the arguments provided to it.
///      This is especially useful for demos or a more extensive
///      "hello world" Docker application.
static POD_IMAGE: &str = "hashicorp/http-echo";
/// Default `hashcorp/http-echo` port.
static POD_PORT: i32 = 5678;
/// Echo finalizer control.
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
///
/// From https://docs.rs/kube-runtime/0.83.0/kube_runtime/finalizer/fn.finalizer.html:
///         In typical usage, if you use finalizer then it should be the only
///         top-level “action” in your applier/Controller’s reconcile function.
pub(crate) async fn reconcile(
    http_echo: Arc<crd::Echo>,
    ctx: Arc<Context>,
) -> Result<Action, Error> {
    // Should be impossible not have a namespace.
    let namespace = http_echo.namespace().unwrap();
    let echo_api: Api<crd::Echo> = Api::namespaced(ctx.client.clone(), &namespace);
    finalizer(&echo_api, ECHO_FINALIZER, http_echo, |event| async {
        match event {
            // XXX: must be idempotent
            Finalizer::Apply(echo) => echo.reconcile(ctx.clone(), POD_IMAGE, POD_PORT).await,
            // XXX: must be idempotent
            Finalizer::Cleanup(echo) => echo.cleanup(ctx.clone()).await,
        }
    })
    .await
    .map_err(|e| Error::ControllerError(e.to_string()))
}

/// Controller::reconciler, `on_error` do this.
pub(crate) fn on_error(_echo: Arc<Echo>, _error: &Error, _ctx: Arc<Context>) -> Action {
    Action::requeue(Duration::from_secs(5 * 60))
}
