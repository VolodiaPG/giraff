import logging
import os
from io import BytesIO
from typing import Optional
from urllib.parse import urlparse

import torch  # type: ignore
from flask import Flask, abort, request  # type: ignore
from opentelemetry import trace  # type: ignore
from opentelemetry.exporter.otlp.proto.grpc.trace_exporter import (  # type: ignore
    OTLPSpanExporter,
)
from opentelemetry.instrumentation.flask import FlaskInstrumentor  # type: ignore
from opentelemetry.sdk.resources import Resource  # type: ignore
from opentelemetry.sdk.trace import TracerProvider  # type: ignore
from opentelemetry.sdk.trace.export import BatchSpanProcessor  # type: ignore
from PIL import Image  # type: ignore
from torchvision import models, transforms  # type: ignore
from waitress import serve  # type: ignore

model_path = os.environ["SQUEEZENET_MODEL"]

resource = {
    "telemetry.sdk.language": "python",
    "service.name": os.environ.get("ID", "dev"),
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
    return response


@app.route("/", methods=["POST"])
def handle():
    with tracer.start_as_current_span("classification"):
        if "file" not in request.files:
            file = request.get_data()
            file = BytesIO(file)
        else:
            file = request.files["file"]

        model = models.squeezenet1_1()  # initialize the model
        model.load_state_dict(torch.load(model_path))

        input_image = Image.open(file)
        preprocess = transforms.Compose(
            [
                transforms.Resize(256),
                transforms.CenterCrop(224),
                transforms.ToTensor(),
                transforms.Normalize(
                    mean=[0.485, 0.456, 0.406], std=[0.229, 0.224, 0.225]
                ),
            ]
        )
        input_tensor = preprocess(input_image)
        input_batch = input_tensor.unsqueeze(
            0
        )  # Create mini batch as expected by model

        # put the model in the eval mode
        model.eval()
        # carryinig out the inference
        out = model(input_batch)
        print("output shape:", out.shape)

        with open("imagenet_classes.txt") as f:
            classes = [line.strip() for line in f.readlines()]

        _, index = torch.max(out, 1)
        percentage = torch.nn.functional.softmax(out, dim=1)[0] * 100
        print(classes[index[0]], percentage[index[0]].item())

        _, indices = torch.sort(out, descending=True)

        return [(classes[idx], percentage[idx].item()) for idx in indices[0][:5]]


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
