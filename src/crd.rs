use crate::controller::Context;
use crate::error::Error;
use kube::runtime::controller::Action;
use kube::{CustomResource, ResourceExt};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tracing::{info, instrument};

/// `HttpEcho` struct `CustomResourceDefinition` specification.
/// This type should be reflected in the `http-echo-crd.yaml` file
/// and it is meant to be with `kube` to create an `Api<HttpEcho>`
/// object.
#[derive(CustomResource, Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[kube(
    group = "test.com",
    version = "v1",
    kind = "Echo",
    plural = "echoes",
    singular = "echo",
    namespaced
)]
pub struct EchoSpec {
    pub replicas: i32,
    pub text: String,
}

impl Echo {
    /// Reconcile `Echo` `CustomResourceDefinition`.
    /// Add `finalizer`, deploy `deployment`, create `service`.
    ///
    /// # Arguments:
    /// - `ctx`: `Controller::Context` with a Kube `Client`.
    /// - `image`: OCI image name.
    /// - `port`: Container port.
    // XXX: `Client` doesn't implement `Debug`.
    #[instrument(skip(ctx))]
    pub async fn reconcile(
        &self,
        ctx: Arc<Context>,
        image: &str,
        port: i32,
    ) -> Result<Action, Error> {
        let client = ctx.client.clone();
        // Should not be possible do not have a namespace.
        let namespace = self.namespace().unwrap();
        let name = self.name_any();

        // Idempotent.
        crate::finalizer::add(client.clone(), &name, &namespace).await?;
        // If already exists, `replace`.
        crate::deployment::deploy(
            client.clone(),
            image,
            port,
            &name,
            self.spec.replicas,
            &self.spec.text,
            &namespace,
        )
        .await?;
        // If already exists, `NoOp`.
        crate::service::create(client, &name, port, &namespace).await?;
        Ok(Action::requeue(Duration::from_secs(5 * 60)))
    }

    // XXX: `Client` doesn't implement `Debug`.
    #[instrument(skip(ctx))]
    pub async fn cleanup(&self, ctx: Arc<Context>) -> Result<Action, Error> {
        let client = ctx.client.clone();
        // Should not be possible do not have a namespace.
        let namespace = self.namespace().unwrap();
        let name = self.name_any();

        info!("Starting deletion procedure of {}", name);
        crate::deployment::delete(client.clone(), &name, &namespace).await?;
        crate::service::delete(client.clone(), &name, &namespace).await?;
        crate::finalizer::clean(client, &name, &namespace).await?;
        Ok(Action::requeue(Duration::from_secs(5 * 60)))
    }
}
