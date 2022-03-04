/* 
 * OpenFaaS API Gateway
 *
 * OpenFaaS API documentation
 *
 * OpenAPI spec version: 0.8.12
 * 
 * Generated by: https://github.com/swagger-api/swagger-codegen.git
 */


#[allow(unused_imports)]
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteFunctionRequest {
  /// Name of deployed function
  #[serde(rename = "functionName")]
  function_name: String
}

impl DeleteFunctionRequest {
  pub fn new(function_name: String) -> DeleteFunctionRequest {
    DeleteFunctionRequest {
      function_name: function_name
    }
  }

  pub fn set_function_name(&mut self, function_name: String) {
    self.function_name = function_name;
  }

  pub fn with_function_name(mut self, function_name: String) -> DeleteFunctionRequest {
    self.function_name = function_name;
    self
  }

  pub fn function_name(&self) -> &String {
    &self.function_name
  }


}


