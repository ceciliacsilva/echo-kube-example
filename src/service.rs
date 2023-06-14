use k8s_openapi::api::core::v1::{Service, ServicePort, ServiceSpec};
use k8s_openapi::apimachinery::pkg::util::intstr::IntOrString;
use kube::api::{DeleteParams, PostParams};
use kube::core::ObjectMeta;
use kube::{Api, Client};
use std::collections::BTreeMap;
use tracing::{info, instrument};

use crate::error::Error;

/// Create a `Expose` service to `HttpEcho` deployment.
///
/// # Arguments:
/// - `client`: A Kubernetes client to be used.
/// - `name`: Name of the `Deployment`.
/// - `namespace`: `Service` namespace.
// NOTE: `Client` doesn't implement `Debug` so `skip(client)`.
#[instrument(skip(client))]
pub async fn create(
    client: Client,
    name: &str,
    port: i32,
    namespace: &str,
) -> Result<Service, Error> {
    let mut labels: BTreeMap<String, String> = BTreeMap::new();
    labels.insert("test.com/app".to_owned(), name.to_owned());

    let service: Service = Service {
        metadata: ObjectMeta {
            name: Some(name.to_owned()),
            ..ObjectMeta::default()
        },
        spec: Some(ServiceSpec {
            type_: Some("NodePort".to_owned()),
            selector: Some(labels.clone()),
            ports: Some(vec![ServicePort {
                port,
                target_port: Some(IntOrString::Int(port)),
                ..ServicePort::default()
            }]),
            ..ServiceSpec::default()
        }),
        ..Service::default()
    };

    let service_api: Api<Service> = Api::namespaced(client, namespace);

    match service_api.get_opt(name).await? {
        Some(service) => {
            //XXX: should I patch existing service?!?
            Ok(service)
        }
        None => {
            // create service
            info!("Creating {name} service");
            service_api.create(&PostParams::default(), &service).await?;
            Ok(service)
        }
    }
}

/// Delete `Expose` HttpEcho `Service`.
///
/// # Arguments:
/// - `client`: Kubernetes client to delete `service`.
/// - `name`: Name of the `Service` to be deleted.
/// - `namespace`: `Service`'s namespace.
// XXX: `Client` doesn't implement `Debug`.
#[instrument(skip(client))]
pub async fn delete(client: Client, name: &str, namespace: &str) -> Result<(), Error> {
    let service_api: Api<Service> = Api::namespaced(client, namespace);

    match service_api.get_opt(name).await? {
        Some(_service) => {
            info!("Deleting {name} service");
            service_api.delete(name, &DeleteParams::default()).await?;
            Ok(())
        }
        None => {
            // create service
            info!("Service {name} does not exists");
            Ok(())
        }
    }
}
