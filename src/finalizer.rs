use crate::crd::Echo;
use crate::error::Error;
use kube::{
    api::{Patch, PatchParams},
    Api, Client,
};
use serde_json::{json, Value};

static ECHO_FINALIZER: &str = "echo.test.com";

/// Adds a `Finalizer` to `HttpEcho` - Kubernetes finalizers.
///
/// From https://kubernetes.io/docs/concepts/overview/working-with-objects/finalizers/:
///       Finalizers alert controllers to clean up resources
///       the deleted object owned.
///
/// # Arguments:
/// - `client`: Kubernetes client.
/// - `name`: CRD resource name.
/// - `namespace`: CRD namespace.
///
/// Note: this should be idempotent.
/// XXX: do I need to check if the `finalizer` already exists?
pub async fn add(client: Client, name: &str, namespace: &str) -> Result<Echo, Error> {
    let api: Api<Echo> = Api::namespaced(client, namespace);
    let finalizer: Value = json!({
        "metadata": {
            "finalizers": [ECHO_FINALIZER],
        }
    });

    let patch: Patch<&Value> = Patch::Merge(&finalizer);
    let http_echo = api.patch(name, &PatchParams::default(), &patch).await?;
    Ok(http_echo)
}

/// Clean all `Finalizers` informing Kubernetes that is safe to delete our CRD.
///
/// # Arguments:
/// - `client`: Kubernetes client.
/// - `name`: CRD resource name.
/// - `namespace`: CRD namespace.
///
/// Note: this should be idempotent.
pub async fn clean(client: Client, name: &str, namespace: &str) -> Result<Echo, Error> {
    let api: Api<Echo> = Api::namespaced(client, namespace);
    let finalizer: Value = json!({
        "metadata": {
            "finalizers": null,
        }
    });

    let patch: Patch<&Value> = Patch::Merge(&finalizer);
    let http_echo = api.patch(name, &PatchParams::default(), &patch).await?;
    Ok(http_echo)
}
