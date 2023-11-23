from flask import Flask, abort, request  # type: ignore
from waitress import serve  # type: ignore
import requests
import logging
import os
from opentelemetry.instrumentation.flask import FlaskInstrumentor
from opentelemetry import trace
from opentelemetry.exporter.otlp.proto.grpc.trace_exporter import OTLPSpanExporter
from opentelemetry.sdk.trace import TracerProvider
from opentelemetry.sdk.trace.export import BatchSpanProcessor
from opentelemetry.instrumentation.requests import RequestsInstrumentor
from opentelemetry.sdk.resources import Resource
import speech_recognition as sr  # type: ignore

resource = {
    "telemetry.sdk.language": "python",
    "service.name": os.environ.get("SLA_ID", "dev"),
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
RequestsInstrumentor().instrument()


NEXT_URL: str = None


@app.route("/", methods=["POST"])
def handle():
    with tracer.start_as_current_span("speech_recognition"):
        if "file" not in request.files:
            abort(400)

        file = request.files["file"]

        finalData = "hello there"
        try:
            r = sr.Recognizer()
            with sr.AudioFile(file) as source:
                audio_data = r.listen(source)
                finalData = r.recognize_vosk(audio_data)
                print("\nThis is the output:", finalData)
        except Exception as e:
            print("Following error was obeserved:", e)
            print("Exiting the code.")
            exit(0)

        if NEXT_URL:
            with tracer.start_as_current_span("forwarding"):
                requests.post(NEXT_URL, finalData)
                return ("", 200)

        return finalData


@app.route("/reconfigure", methods=["POST"])
def reconfigure():
    global NEXT_URL
    req = request.get_json()
    if "nextFunctionUrl" not in req:
        abort(400, description=f"nextFunctionUrl not in {req}")
    NEXT_URL = req["nextFunctionUrl"]
    print(NEXT_URL)
    return ("", 200)


if __name__ == "__main__":
    serve(app, host="0.0.0.0", port=5000)
