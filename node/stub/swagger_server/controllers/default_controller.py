import connexion
import six

from swagger_server.models.binary import Binary  # noqa: E501
from swagger_server.models.delete_function_request import DeleteFunctionRequest  # noqa: E501
from swagger_server.models.function_definition import FunctionDefinition  # noqa: E501
from swagger_server.models.function_list_entry import FunctionListEntry  # noqa: E501
from swagger_server.models.info import Info  # noqa: E501
from swagger_server.models.log_entry import LogEntry  # noqa: E501
from swagger_server.models.secret import Secret  # noqa: E501
from swagger_server.models.secret_name import SecretName  # noqa: E501
from swagger_server import util


def async_function_function_name_post(functionName, input=None):  # noqa: E501
    """Invoke a function asynchronously in OpenFaaS

    See https://docs.openfaas.com/reference/async/. # noqa: E501

    :param functionName: Function name
    :type functionName: str
    :param input: (Optional) data to pass to function
    :type input: str

    :rtype: None
    """
    return 'do some magic!'


def function_function_name_post(functionName, input=None):  # noqa: E501
    """Invoke a function defined in OpenFaaS

     # noqa: E501

    :param functionName: Function name
    :type functionName: str
    :param input: (Optional) data to pass to function
    :type input: str

    :rtype: None
    """
    return 'do some magic!'


def healthz_get():  # noqa: E501
    """Healthcheck

     # noqa: E501


    :rtype: None
    """
    return 'do some magic!'


def system_alert_post(body=None):  # noqa: E501
    """Event-sink for AlertManager, for auto-scaling

    Internal use for AlertManager, requires valid AlertManager alert JSON # noqa: E501

    :param body: Incoming alert
    :type body: 

    :rtype: None
    """
    return 'do some magic!'


def system_function_function_name_get(functionName):  # noqa: E501
    """Get a summary of an OpenFaaS function

     # noqa: E501

    :param functionName: Function name
    :type functionName: str

    :rtype: FunctionListEntry
    """
    return 'do some magic!'


def system_functions_delete(body):  # noqa: E501
    """Remove a deployed function.

     # noqa: E501

    :param body: Function to delete
    :type body: dict | bytes

    :rtype: None
    """
    if connexion.request.is_json:
        body = DeleteFunctionRequest.from_dict(connexion.request.get_json())  # noqa: E501
    return 'do some magic!'


def system_functions_get():  # noqa: E501
    """Get a list of deployed functions with: stats and image digest

     # noqa: E501


    :rtype: List[FunctionListEntry]
    """
    return 'do some magic!'


def system_functions_post(body):  # noqa: E501
    """Deploy a new function.

     # noqa: E501

    :param body: Function to deploy
    :type body: dict | bytes

    :rtype: None
    """
    if connexion.request.is_json:
        body = FunctionDefinition.from_dict(connexion.request.get_json())  # noqa: E501
    return 'do some magic!'


def system_functions_put(body):  # noqa: E501
    """Update a function.

     # noqa: E501

    :param body: Function to update
    :type body: dict | bytes

    :rtype: None
    """
    if connexion.request.is_json:
        body = FunctionDefinition.from_dict(connexion.request.get_json())  # noqa: E501
    return 'do some magic!'


def system_info_get():  # noqa: E501
    """Get info such as provider version number and provider orchestrator

     # noqa: E501


    :rtype: Info
    """
    return 'do some magic!'


def system_logs_get(name, since=None, tail=None, follow=None):  # noqa: E501
    """Get a stream of the logs for a specific function

     # noqa: E501

    :param name: Function name
    :type name: str
    :param since: Only return logs after a specific date (RFC3339)
    :type since: str
    :param tail: Sets the maximum number of log messages to return, &lt;&#x3D;0 means unlimited
    :type tail: int
    :param follow: When true, the request will stream logs until the request timeout
    :type follow: bool

    :rtype: LogEntry
    """
    return 'do some magic!'


def system_scale_function_function_name_post(functionName, input=None):  # noqa: E501
    """Scale a function

     # noqa: E501

    :param functionName: Function name
    :type functionName: str
    :param input: Function to scale plus replica count
    :type input: str

    :rtype: None
    """
    return 'do some magic!'


def system_secrets_delete(body):  # noqa: E501
    """Remove a secret.

     # noqa: E501

    :param body: Secret to delete
    :type body: dict | bytes

    :rtype: None
    """
    if connexion.request.is_json:
        body = SecretName.from_dict(connexion.request.get_json())  # noqa: E501
    return 'do some magic!'


def system_secrets_get():  # noqa: E501
    """Get a list of secret names and metadata from the provider

     # noqa: E501


    :rtype: SecretName
    """
    return 'do some magic!'


def system_secrets_post(body):  # noqa: E501
    """Create a new secret.

     # noqa: E501

    :param body: A new secret to create
    :type body: dict | bytes

    :rtype: None
    """
    if connexion.request.is_json:
        body = Secret.from_dict(connexion.request.get_json())  # noqa: E501
    return 'do some magic!'


def system_secrets_put(body):  # noqa: E501
    """Update a secret.

     # noqa: E501

    :param body: Secret to update
    :type body: dict | bytes

    :rtype: None
    """
    if connexion.request.is_json:
        body = Secret.from_dict(connexion.request.get_json())  # noqa: E501
    return 'do some magic!'
