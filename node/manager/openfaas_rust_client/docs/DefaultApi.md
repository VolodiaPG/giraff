# \DefaultApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**async_function_function_name_post**](DefaultApi.md#async_function_function_name_post) | **Post** /async-function/{functionName} | Invoke a function asynchronously in OpenFaaS
[**function_function_name_post**](DefaultApi.md#function_function_name_post) | **Post** /function/{functionName} | Invoke a function defined in OpenFaaS
[**healthz_get**](DefaultApi.md#healthz_get) | **Get** /healthz | Healthcheck
[**system_alert_post**](DefaultApi.md#system_alert_post) | **Post** /system/alert | Event-sink for AlertManager, for auto-scaling
[**system_function_function_name_get**](DefaultApi.md#system_function_function_name_get) | **Get** /system/function/{functionName} | Get a summary of an OpenFaaS function
[**system_functions_delete**](DefaultApi.md#system_functions_delete) | **Delete** /system/functions | Remove a deployed function.
[**system_functions_get**](DefaultApi.md#system_functions_get) | **Get** /system/functions | Get a list of deployed functions with: stats and image digest
[**system_functions_post**](DefaultApi.md#system_functions_post) | **Post** /system/functions | Deploy a new function.
[**system_functions_put**](DefaultApi.md#system_functions_put) | **Put** /system/functions | Update a function.
[**system_info_get**](DefaultApi.md#system_info_get) | **Get** /system/info | Get info such as provider version number and provider orchestrator
[**system_logs_get**](DefaultApi.md#system_logs_get) | **Get** /system/logs | Get a stream of the logs for a specific function
[**system_scale_function_function_name_post**](DefaultApi.md#system_scale_function_function_name_post) | **Post** /system/scale-function/{functionName} | Scale a function
[**system_secrets_delete**](DefaultApi.md#system_secrets_delete) | **Delete** /system/secrets | Remove a secret.
[**system_secrets_get**](DefaultApi.md#system_secrets_get) | **Get** /system/secrets | Get a list of secret names and metadata from the provider
[**system_secrets_post**](DefaultApi.md#system_secrets_post) | **Post** /system/secrets | Create a new secret.
[**system_secrets_put**](DefaultApi.md#system_secrets_put) | **Put** /system/secrets | Update a secret.


# **async_function_function_name_post**
> async_function_function_name_post(function_name, optional)
Invoke a function asynchronously in OpenFaaS

See https://docs.openfaas.com/reference/async/.

### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
  **function_name** | **String**| Function name | 
 **optional** | **map[string]interface{}** | optional parameters | nil if no parameters

### Optional Parameters
Optional parameters are passed through a map[string]interface{}.

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **function_name** | **String**| Function name | 
 **input** | **Vec&lt;u8&gt;**| (Optional) data to pass to function | 

### Return type

 (empty response body)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **function_function_name_post**
> function_function_name_post(function_name, optional)
Invoke a function defined in OpenFaaS

### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
  **function_name** | **String**| Function name | 
 **optional** | **map[string]interface{}** | optional parameters | nil if no parameters

### Optional Parameters
Optional parameters are passed through a map[string]interface{}.

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **function_name** | **String**| Function name | 
 **input** | **Vec&lt;u8&gt;**| (Optional) data to pass to function | 

### Return type

 (empty response body)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **healthz_get**
> healthz_get()
Healthcheck

### Required Parameters
This endpoint does not need any parameter.

### Return type

 (empty response body)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **system_alert_post**
> system_alert_post(optional)
Event-sink for AlertManager, for auto-scaling

Internal use for AlertManager, requires valid AlertManager alert JSON

### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **optional** | **map[string]interface{}** | optional parameters | nil if no parameters

### Optional Parameters
Optional parameters are passed through a map[string]interface{}.

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **body** | [**Value**](Value.md)| Incoming alert | 

### Return type

 (empty response body)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **system_function_function_name_get**
> ::models::FunctionListEntry system_function_function_name_get(function_name)
Get a summary of an OpenFaaS function

### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
  **function_name** | **String**| Function name | 

### Return type

[**::models::FunctionListEntry**](FunctionListEntry.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **system_functions_delete**
> system_functions_delete(body)
Remove a deployed function.



### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
  **body** | [**DeleteFunctionRequest**](DeleteFunctionRequest.md)| Function to delete | 

### Return type

 (empty response body)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **system_functions_get**
> Vec<::models::FunctionListEntry> system_functions_get()
Get a list of deployed functions with: stats and image digest

### Required Parameters
This endpoint does not need any parameter.

### Return type

[**Vec<::models::FunctionListEntry>**](FunctionListEntry.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **system_functions_post**
> system_functions_post(body)
Deploy a new function.



### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
  **body** | [**FunctionDefinition**](FunctionDefinition.md)| Function to deploy | 

### Return type

 (empty response body)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **system_functions_put**
> system_functions_put(body)
Update a function.



### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
  **body** | [**FunctionDefinition**](FunctionDefinition.md)| Function to update | 

### Return type

 (empty response body)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **system_info_get**
> ::models::Info system_info_get()
Get info such as provider version number and provider orchestrator

### Required Parameters
This endpoint does not need any parameter.

### Return type

[**::models::Info**](Info.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **system_logs_get**
> ::models::LogEntry system_logs_get(name, optional)
Get a stream of the logs for a specific function

### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
  **name** | **String**| Function name | 
 **optional** | **map[string]interface{}** | optional parameters | nil if no parameters

### Optional Parameters
Optional parameters are passed through a map[string]interface{}.

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **name** | **String**| Function name | 
 **since** | **String**| Only return logs after a specific date (RFC3339) | 
 **tail** | **i32**| Sets the maximum number of log messages to return, &lt;&#x3D;0 means unlimited | 
 **follow** | **bool**| When true, the request will stream logs until the request timeout | 

### Return type

[**::models::LogEntry**](LogEntry.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/x-ndjson

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **system_scale_function_function_name_post**
> system_scale_function_function_name_post(function_name, optional)
Scale a function

### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
  **function_name** | **String**| Function name | 
 **optional** | **map[string]interface{}** | optional parameters | nil if no parameters

### Optional Parameters
Optional parameters are passed through a map[string]interface{}.

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **function_name** | **String**| Function name | 
 **input** | **Vec&lt;u8&gt;**| Function to scale plus replica count | 

### Return type

 (empty response body)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **system_secrets_delete**
> system_secrets_delete(body)
Remove a secret.



### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
  **body** | [**SecretName**](SecretName.md)| Secret to delete | 

### Return type

 (empty response body)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **system_secrets_get**
> ::models::SecretName system_secrets_get()
Get a list of secret names and metadata from the provider

### Required Parameters
This endpoint does not need any parameter.

### Return type

[**::models::SecretName**](SecretName.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **system_secrets_post**
> system_secrets_post(body)
Create a new secret.



### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
  **body** | [**Secret**](Secret.md)| A new secret to create | 

### Return type

 (empty response body)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **system_secrets_put**
> system_secrets_put(body)
Update a secret.



### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
  **body** | [**Secret**](Secret.md)| Secret to update | 

### Return type

 (empty response body)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

