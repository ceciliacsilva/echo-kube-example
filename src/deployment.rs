use k8s_openapi::api::apps::v1::Deployment;
use k8s_openapi::api::apps::v1::DeploymentSpec;
use k8s_openapi::api::core::v1::{Container, ContainerPort, PodSpec, PodTemplateSpec};
use k8s_openapi::apimachinery::pkg::apis::meta::v1::LabelSelector;
use kube::api::{DeleteParams, ObjectMeta, PostParams};
use kube::{Api, Client};
use serde_json::json;
use std::collections::BTreeMap;
use tracing::info;

use crate::error::Error;

/// Creates a new `deployment` with `n` replicas with the ` hashicorp/http-echo`
/// OCI image.
///
/// # Arguments
/// - `client`: A Kubernetes client to be used.
/// - `name`: Name of the `deployment`.
/// - `replicas`: Number of pod/replicas to be created.
/// - `namespace`: `Deployment` namespace.
pub async fn deploy(
    client: Client,
    image_name: &str,
    port: i32,
    name: &str,
    replicas: i32,
    text: &str,
    namespace: &str,
) -> Result<Deployment, Error> {
    let mut labels: BTreeMap<String, String> = BTreeMap::new();
    labels.insert("test.com/app".to_owned(), name.to_owned());

    let deployment: Deployment = Deployment {
        metadata: ObjectMeta {
            name: Some(name.to_owned()),
            namespace: Some(namespace.to_owned()),
            labels: Some(labels.clone()),
            ..ObjectMeta::default()
        },
        spec: Some(DeploymentSpec {
            replicas: Some(replicas),
            selector: LabelSelector {
                match_expressions: None,
                match_labels: Some(labels.clone()),
            },
            template: PodTemplateSpec {
                spec: Some(PodSpec {
                    containers: vec![Container {
                        name: name.to_owned(),
                        image: Some(image_name.to_owned()),
                        ports: Some(vec![ContainerPort {
                            container_port: port,
                            ..ContainerPort::default()
                        }]),
                        args: Some(vec![format!(r#"--text="{}""#, text.to_owned())]),
                        ..Container::default()
                    }],
                    ..PodSpec::default()
                }),
                metadata: Some(ObjectMeta {
                    labels: Some(labels),
                    ..ObjectMeta::default()
                }),
            },
            ..DeploymentSpec::default()
        }),
        ..Deployment::default()
    };

    let deployment_api: Api<Deployment> = Api::namespaced(client, namespace);
    info!("Creating a {name} deployment");
    deployment_api
        .create(&PostParams::default(), &deployment)
        .await?;
    Ok(deployment)
}

/// Delete a `Deployment` called `name` inside the `namespace`.
///
/// # Arguments:
/// - `client` - A Kubernetes `client` to be used.
/// - `name` - `Deployment` to be deleted.
/// - `namespace` - `Deployment`'s namespace.
pub async fn delete(client: Client, name: &str, namespace: &str) -> Result<(), Error> {
    let api: Api<Deployment> = Api::namespaced(client, &namespace);
    info!("Deleting a {name} deployment");
    api.delete(name, &DeleteParams::default()).await?;
    Ok(())
}
