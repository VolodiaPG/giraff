# coding: utf-8

from __future__ import absolute_import

from flask import json
from six import BytesIO

from swagger_server.models.binary import Binary  # noqa: E501
from swagger_server.models.delete_function_request import DeleteFunctionRequest  # noqa: E501
from swagger_server.models.function_definition import FunctionDefinition  # noqa: E501
from swagger_server.models.function_list_entry import FunctionListEntry  # noqa: E501
from swagger_server.models.info import Info  # noqa: E501
from swagger_server.models.log_entry import LogEntry  # noqa: E501
from swagger_server.models.secret import Secret  # noqa: E501
from swagger_server.models.secret_name import SecretName  # noqa: E501
from swagger_server.test import BaseTestCase


class TestDefaultController(BaseTestCase):
    """DefaultController integration test stubs"""

    def test_async_function_function_name_post(self):
        """Test case for async_function_function_name_post

        Invoke a function asynchronously in OpenFaaS
        """
        input = Binary()
        response = self.client.open(
            '//async-function/{functionName}'.format(functionName='functionName_example'),
            method='POST',
            data=json.dumps(input),
            content_type='application/json')
        self.assert200(response,
                       'Response body is : ' + response.data.decode('utf-8'))

    def test_function_function_name_post(self):
        """Test case for function_function_name_post

        Invoke a function defined in OpenFaaS
        """
        input = Binary()
        response = self.client.open(
            '//function/{functionName}'.format(functionName='functionName_example'),
            method='POST',
            data=json.dumps(input),
            content_type='application/json')
        self.assert200(response,
                       'Response body is : ' + response.data.decode('utf-8'))

    def test_healthz_get(self):
        """Test case for healthz_get

        Healthcheck
        """
        response = self.client.open(
            '//healthz',
            method='GET')
        self.assert200(response,
                       'Response body is : ' + response.data.decode('utf-8'))

    def test_system_alert_post(self):
        """Test case for system_alert_post

        Event-sink for AlertManager, for auto-scaling
        """
        body = None
        response = self.client.open(
            '//system/alert',
            method='POST',
            data=json.dumps(body),
            content_type='application/json')
        self.assert200(response,
                       'Response body is : ' + response.data.decode('utf-8'))

    def test_system_function_function_name_get(self):
        """Test case for system_function_function_name_get

        Get a summary of an OpenFaaS function
        """
        response = self.client.open(
            '//system/function/{functionName}'.format(functionName='functionName_example'),
            method='GET')
        self.assert200(response,
                       'Response body is : ' + response.data.decode('utf-8'))

    def test_system_functions_delete(self):
        """Test case for system_functions_delete

        Remove a deployed function.
        """
        body = DeleteFunctionRequest()
        response = self.client.open(
            '//system/functions',
            method='DELETE',
            data=json.dumps(body),
            content_type='application/json')
        self.assert200(response,
                       'Response body is : ' + response.data.decode('utf-8'))

    def test_system_functions_get(self):
        """Test case for system_functions_get

        Get a list of deployed functions with: stats and image digest
        """
        response = self.client.open(
            '//system/functions',
            method='GET',
            content_type='application/json')
        self.assert200(response,
                       'Response body is : ' + response.data.decode('utf-8'))

    def test_system_functions_post(self):
        """Test case for system_functions_post

        Deploy a new function.
        """
        body = FunctionDefinition()
        response = self.client.open(
            '//system/functions',
            method='POST',
            data=json.dumps(body),
            content_type='application/json')
        self.assert200(response,
                       'Response body is : ' + response.data.decode('utf-8'))

    def test_system_functions_put(self):
        """Test case for system_functions_put

        Update a function.
        """
        body = FunctionDefinition()
        response = self.client.open(
            '//system/functions',
            method='PUT',
            data=json.dumps(body),
            content_type='application/json')
        self.assert200(response,
                       'Response body is : ' + response.data.decode('utf-8'))

    def test_system_info_get(self):
        """Test case for system_info_get

        Get info such as provider version number and provider orchestrator
        """
        response = self.client.open(
            '//system/info',
            method='GET')
        self.assert200(response,
                       'Response body is : ' + response.data.decode('utf-8'))

    def test_system_logs_get(self):
        """Test case for system_logs_get

        Get a stream of the logs for a specific function
        """
        query_string = [('name', 'name_example'),
                        ('since', 'since_example'),
                        ('tail', 56),
                        ('follow', true)]
        response = self.client.open(
            '//system/logs',
            method='GET',
            query_string=query_string)
        self.assert200(response,
                       'Response body is : ' + response.data.decode('utf-8'))

    def test_system_scale_function_function_name_post(self):
        """Test case for system_scale_function_function_name_post

        Scale a function
        """
        input = Binary()
        response = self.client.open(
            '//system/scale-function/{functionName}'.format(functionName='functionName_example'),
            method='POST',
            data=json.dumps(input),
            content_type='application/json')
        self.assert200(response,
                       'Response body is : ' + response.data.decode('utf-8'))

    def test_system_secrets_delete(self):
        """Test case for system_secrets_delete

        Remove a secret.
        """
        body = SecretName()
        response = self.client.open(
            '//system/secrets',
            method='DELETE',
            data=json.dumps(body),
            content_type='application/json')
        self.assert200(response,
                       'Response body is : ' + response.data.decode('utf-8'))

    def test_system_secrets_get(self):
        """Test case for system_secrets_get

        Get a list of secret names and metadata from the provider
        """
        response = self.client.open(
            '//system/secrets',
            method='GET',
            content_type='application/json')
        self.assert200(response,
                       'Response body is : ' + response.data.decode('utf-8'))

    def test_system_secrets_post(self):
        """Test case for system_secrets_post

        Create a new secret.
        """
        body = Secret()
        response = self.client.open(
            '//system/secrets',
            method='POST',
            data=json.dumps(body),
            content_type='application/json')
        self.assert200(response,
                       'Response body is : ' + response.data.decode('utf-8'))

    def test_system_secrets_put(self):
        """Test case for system_secrets_put

        Update a secret.
        """
        body = Secret()
        response = self.client.open(
            '//system/secrets',
            method='PUT',
            data=json.dumps(body),
            content_type='application/json')
        self.assert200(response,
                       'Response body is : ' + response.data.decode('utf-8'))


if __name__ == '__main__':
    import unittest
    unittest.main()
