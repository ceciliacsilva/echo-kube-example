/// All `HttpEcho`-operator errors.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Errors coming from `Kubernetes`.
    #[error("Kubernetes reported error: {source}")]
    Kube {
        #[from]
        source: kube::Error,
    },

    #[error("Serde_json reported error: {source}")]
    SerdeJson {
        #[from]
        source: serde_json::Error,
    },

    /// Reconciliation error.
    #[error("Controller reported error:{0}")]
    ControllerError(String),
}
