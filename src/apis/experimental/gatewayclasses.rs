// WARNING: generated by kopium - manual changes will be overwritten
// kopium command: kopium -Af -
// kopium version: 0.16.1

use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Serialize, Deserialize};

/// Spec defines the desired state of GatewayClass.
#[derive(CustomResource, Serialize, Deserialize, Clone, Debug, JsonSchema)]
#[kube(group = "gateway.networking.k8s.io", version = "v1", kind = "GatewayClass", plural = "gatewayclasses")]
#[kube(status = "GatewayClassStatus")]
pub struct GatewayClassSpec {
    /// ControllerName is the name of the controller that is managing Gateways of this class. The value of this field MUST be a domain prefixed path. 
    ///  Example: "example.net/gateway-controller". 
    ///  This field is not mutable and cannot be empty. 
    ///  Support: Core
    #[serde(rename = "controllerName")]
    pub controller_name: String,
    /// Description helps describe a GatewayClass with more details.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// ParametersRef is a reference to a resource that contains the configuration parameters corresponding to the GatewayClass. This is optional if the controller does not require any additional configuration. 
    ///  ParametersRef can reference a standard Kubernetes resource, i.e. ConfigMap, or an implementation-specific custom resource. The resource can be cluster-scoped or namespace-scoped. 
    ///  If the referent cannot be found, the GatewayClass's "InvalidParameters" status condition will be true. 
    ///  Support: Implementation-specific
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "parametersRef")]
    pub parameters_ref: Option<GatewayClassParametersRef>,
}

/// ParametersRef is a reference to a resource that contains the configuration parameters corresponding to the GatewayClass. This is optional if the controller does not require any additional configuration. 
///  ParametersRef can reference a standard Kubernetes resource, i.e. ConfigMap, or an implementation-specific custom resource. The resource can be cluster-scoped or namespace-scoped. 
///  If the referent cannot be found, the GatewayClass's "InvalidParameters" status condition will be true. 
///  Support: Implementation-specific
#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
pub struct GatewayClassParametersRef {
    /// Group is the group of the referent.
    pub group: String,
    /// Kind is kind of the referent.
    pub kind: String,
    /// Name is the name of the referent.
    pub name: String,
    /// Namespace is the namespace of the referent. This field is required when referring to a Namespace-scoped resource and MUST be unset when referring to a Cluster-scoped resource.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub namespace: Option<String>,
}

/// Status defines the current state of GatewayClass. 
///  Implementations MUST populate status on all GatewayClass resources which specify their controller name.
#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
pub struct GatewayClassStatus {
    /// Conditions is the current status from the controller for this GatewayClass. 
    ///  Controllers should prefer to publish conditions using values of GatewayClassConditionType for the type of each Condition.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub conditions: Option<Vec<GatewayClassStatusConditions>>,
    /// SupportedFeatures is the set of features the GatewayClass support. It MUST be sorted in ascending alphabetical order. 
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "supportedFeatures")]
    pub supported_features: Option<Vec<String>>,
}

/// Condition contains details for one aspect of the current state of this API Resource. --- This struct is intended for direct use as an array at the field path .status.conditions.  For example, 
///  type FooStatus struct{ // Represents the observations of a foo's current state. // Known .status.conditions.type are: "Available", "Progressing", and "Degraded" // +patchMergeKey=type // +patchStrategy=merge // +listType=map // +listMapKey=type Conditions []metav1.Condition `json:"conditions,omitempty" patchStrategy:"merge" patchMergeKey:"type" protobuf:"bytes,1,rep,name=conditions"` 
///  // other fields }
#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
pub struct GatewayClassStatusConditions {
    /// lastTransitionTime is the last time the condition transitioned from one status to another. This should be when the underlying condition changed.  If that is not known, then using the time when the API field changed is acceptable.
    #[serde(rename = "lastTransitionTime")]
    pub last_transition_time: String,
    /// message is a human readable message indicating details about the transition. This may be an empty string.
    pub message: String,
    /// observedGeneration represents the .metadata.generation that the condition was set based upon. For instance, if .metadata.generation is currently 12, but the .status.conditions[x].observedGeneration is 9, the condition is out of date with respect to the current state of the instance.
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "observedGeneration")]
    pub observed_generation: Option<i64>,
    /// reason contains a programmatic identifier indicating the reason for the condition's last transition. Producers of specific condition types may define expected values and meanings for this field, and whether the values are considered a guaranteed API. The value should be a CamelCase string. This field may not be empty.
    pub reason: String,
    /// status of the condition, one of True, False, Unknown.
    pub status: GatewayClassStatusConditionsStatus,
    /// type of condition in CamelCase or in foo.example.com/CamelCase. --- Many .condition.type values are consistent across resources like Available, but because arbitrary conditions can be useful (see .node.status.conditions), the ability to deconflict is important. The regex it matches is (dns1123SubdomainFmt/)?(qualifiedNameFmt)
    #[serde(rename = "type")]
    pub r#type: String,
}

/// Condition contains details for one aspect of the current state of this API Resource. --- This struct is intended for direct use as an array at the field path .status.conditions.  For example, 
///  type FooStatus struct{ // Represents the observations of a foo's current state. // Known .status.conditions.type are: "Available", "Progressing", and "Degraded" // +patchMergeKey=type // +patchStrategy=merge // +listType=map // +listMapKey=type Conditions []metav1.Condition `json:"conditions,omitempty" patchStrategy:"merge" patchMergeKey:"type" protobuf:"bytes,1,rep,name=conditions"` 
///  // other fields }
#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
pub enum GatewayClassStatusConditionsStatus {
    True,
    False,
    Unknown,
}

