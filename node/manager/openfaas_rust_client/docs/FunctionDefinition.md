# FunctionDefinition

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**service** | **String** | Name of deployed function | [default to null]
**network** | **String** | Network, usually func_functions for Swarm (deprecated) | [optional] [default to null]
**image** | **String** | Docker image in accessible registry | [default to null]
**env_process** | **String** | Process for watchdog to fork | [default to null]
**env_vars** | **::std::collections::HashMap<String, String>** | Overrides to environmental variables | [optional] [default to null]
**constraints** | **Vec<String>** |  | [optional] [default to null]
**labels** | **::std::collections::HashMap<String, String>** | A map of labels for making scheduling or routing decisions | [optional] [default to null]
**annotations** | **::std::collections::HashMap<String, String>** | A map of annotations for management, orchestration, events and build tasks | [optional] [default to null]
**secrets** | **Vec<String>** |  | [optional] [default to null]
**registry_auth** | **String** | Private registry base64-encoded basic auth (as present in ~/.docker/config.json) | [optional] [default to null]
**limits** | [***Value**](Value.md) |  | [optional] [default to null]
**requests** | [***Value**](Value.md) |  | [optional] [default to null]
**read_only_root_filesystem** | **bool** | Make the root filesystem of the function read-only | [optional] [default to null]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


