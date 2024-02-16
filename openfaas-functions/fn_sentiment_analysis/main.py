import logging
import os
from typing import Optional
from urllib.parse import urlparse

from flask import Flask, abort, request  # type: ignore
from opentelemetry import trace  # type: ignore
from opentelemetry.exporter.otlp.proto.grpc.trace_exporter import (  # type: ignore
    OTLPSpanExporter,
)
from opentelemetry.instrumentation.flask import FlaskInstrumentor  # type: ignore
from opentelemetry.sdk.resources import Resource  # type: ignore
from opentelemetry.sdk.trace import TracerProvider  # type: ignore
from opentelemetry.sdk.trace.export import BatchSpanProcessor  # type: ignore
from textblob import TextBlob  # type: ignore
from waitress import serve  # type: ignore

my_name = os.environ.get("NAME", "dev")
my_sla_id = os.environ.get("ID", "dev")
resource = {
    "telemetry.sdk.language": "python",
    "service.name": my_sla_id
}
resource = Resource.create(resource)


logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)


otel_exporter_otlp_endpoint = os.environ.get("OTEL_EXPORTER_OTLP_ENDPOINT_FUNCTION")
print(otel_exporter_otlp_endpoint)

provider = TracerProvider(resource=resource)
exporter = OTLPSpanExporter(endpoint=otel_exporter_otlp_endpoint)
processor = BatchSpanProcessor(exporter)
provider.add_span_processor(processor)
trace.set_tracer_provider(provider)
tracer = trace.get_tracer(__name__)


app = Flask(__name__)
FlaskInstrumentor().instrument_app(app)


NEXT_URL: Optional[str] = None


@app.after_request
def add_headers(response):
    if NEXT_URL:
        response.headers["GIRAFF-Redirect"] = NEXT_URL
        parsed_url = urlparse(NEXT_URL)
        hostname = parsed_url.hostname
        response.headers["GIRAFF-Redirect-Proxy"] = f"http://{hostname}:3128/"
        response.headers["GIRAFF-Tags"] = my_name
        response.headers["GIRAFF-SLA-ID"] = my_sla_id
    return response


@app.route("/", methods=["POST"])
def handle():
    with tracer.start_as_current_span("sentiment analysis"):
        text = request.get_json()
        if "text" not in text:
            logger.error("text field in json required")
            abort(400, "text field in json required")
        blob = TextBlob(text["text"])
        res = {"polarity": 0.0, "subjectivity": 0.0}

        for sentence in blob.sentences:
            res["subjectivity"] = res["subjectivity"] + sentence.sentiment.subjectivity
            res["polarity"] = res["polarity"] + sentence.sentiment.polarity

        total = len(blob.sentences)

        res["sentence_count"] = total
        res["polarity"] = res["polarity"] / total
        res["subjectivity"] = res["subjectivity"] / total

        return res


@app.route("/reconfigure", methods=["POST"])
def reconfigure():
    global NEXT_URL
    req = request.get_json()
    if "nextFunctionUrl" not in req:
        abort(400, description=f"nextFunctionUrl not in {req}")
    NEXT_URL = req["nextFunctionUrl"]
    print(NEXT_URL)
    return ("", 200)

@app.route("/health", methods=["GET"])
def health():
    return ("", 200)

if __name__ == "__main__":
    serve(app, host="0.0.0.0", port=5000)
