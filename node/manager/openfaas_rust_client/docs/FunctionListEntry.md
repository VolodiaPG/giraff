# FunctionListEntry

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**name** | **String** | The name of the function | [default to null]
**image** | **String** | The fully qualified docker image name of the function | [default to null]
**invocation_count** | **f32** | The amount of invocations for the specified function | [default to null]
**replicas** | **f32** | The current minimal ammount of replicas | [default to null]
**available_replicas** | **f32** | The current available amount of replicas | [default to null]
**env_process** | **String** | Process for watchdog to fork | [default to null]
**labels** | **::std::collections::HashMap<String, String>** | A map of labels for making scheduling or routing decisions | [default to null]
**annotations** | **::std::collections::HashMap<String, String>** | A map of annotations for management, orchestration, events and build tasks | [optional] [default to null]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


