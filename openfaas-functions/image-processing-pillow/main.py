from io import BytesIO
import logging
import os
from flask import Flask, abort, request, send_file  # type: ignore
from PIL import Image  # type: ignore
from waitress import serve  # type: ignore
import requests
from opentelemetry.instrumentation.flask import FlaskInstrumentor
from opentelemetry import trace
from opentelemetry.exporter.otlp.proto.grpc.trace_exporter import OTLPSpanExporter
from opentelemetry.sdk.trace import TracerProvider
from opentelemetry.sdk.trace.export import BatchSpanProcessor
from opentelemetry.instrumentation.requests import RequestsInstrumentor
from opentelemetry.sdk.resources import Resource


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


def serve_pil_image(pil_img):
    img_io = BytesIO()
    pil_img.save(img_io, "JPEG", quality=70)
    img_io.seek(0)
    return send_file(img_io, mimetype="image/jpeg")


@app.route("/", methods=["POST"])
def handle():
    global NEXT_URL
    with tracer.start_as_current_span("pillow_image"):
        if "file" not in request.files:
            logger.error(
                "file is not embedded in the request (file not in request.files)"
            )
            abort(
                400,
                description="file is not embedded in the request (file not in request.files)",
            )

        file = request.files["file"]

        im = Image.open(file)

        resized_im = im.resize((int(256 * im.width / im.height), 256))
        if NEXT_URL:
            with tracer.start_as_current_span("forwarding"):
                img_io = BytesIO()
                resized_im.save(img_io, "JPEG", quality=70)
                img_io.seek(0)
                files = {"file": img_io}
                requests.post(NEXT_URL, files)
                return ("", 200)

        return serve_pil_image(resized_im)


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
