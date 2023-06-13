/// All `HttpEcho`-operator errors.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Errors coming from `Kubernetes`.
    #[error("Kubernetes reported error: {source}")]
    KubeError {
        #[from]
        source: kube::Error,
    },

    #[error("Serde_json reported error: {source}")]
    SerdeJsonError {
        #[from]
        source: serde_json::Error,
    },

    /// Input errors.
    #[error("Invalid HttpEcho CRD:{0}")]
    InputError(String),
}
