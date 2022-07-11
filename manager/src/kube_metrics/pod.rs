use k8s_openapi::apimachinery::pkg::{api::resource::Quantity, apis::meta::v1::ObjectMeta};

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
pub struct PodMetricsContainer {
    pub name:  String,
    pub usage: PodMetricsContainerUsage,
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
pub struct PodMetricsContainerUsage {
    /// https://kubernetes.io/docs/concepts/configuration/manage-resources-containers/#meaning-of-cpu
    pub cpu:    Quantity,
    pub memory: Quantity,
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
pub struct PodMetrics {
    pub metadata:   ObjectMeta,
    pub timestamp:  String,
    pub window:     String,
    pub containers: Vec<PodMetricsContainer>,
}

impl k8s_openapi::Resource for PodMetrics {
    type Scope = k8s_openapi::NamespaceResourceScope;

    const API_VERSION: &'static str = "metrics.k8s.io/v1beta1";
    const GROUP: &'static str = "metrics.k8s.io";
    const KIND: &'static str = "pod";
    const URL_PATH_SEGMENT: &'static str = "pods";
    const VERSION: &'static str = "v1beta1";
}

impl k8s_openapi::Metadata for PodMetrics {
    type Ty = ObjectMeta;

    fn metadata(&self) -> &Self::Ty { &self.metadata }

    fn metadata_mut(&mut self) -> &mut Self::Ty { &mut self.metadata }
}
