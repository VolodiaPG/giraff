import json
import logging
import math
import os
import socket
from io import BytesIO
from threading import Semaphore
from typing import Optional
from urllib.parse import urlparse

import psutil
from flask import Flask, abort, request  # type: ignore
from opentelemetry import trace  # type: ignore
from opentelemetry.exporter.otlp.proto.grpc.trace_exporter import (  # type: ignore
    OTLPSpanExporter,
)
from opentelemetry.instrumentation.flask import FlaskInstrumentor  # type: ignore
from opentelemetry.sdk.resources import Resource  # type: ignore
from opentelemetry.sdk.trace import TracerProvider  # type: ignore
from opentelemetry.sdk.trace.export import BatchSpanProcessor  # type: ignore
from speech_recognition import AudioFile, Recognizer  # type: ignore
from waitress import serve  # type: ignore

FREE_MEM_FOR_MODEL = 350 * 1024 * 1024  # MB


class Pool:
    def __init__(self) -> None:
        self.pool = []
        self.busy = Semaphore()
        nb = math.floor(psutil.virtual_memory().free / FREE_MEM_FOR_MODEL)
        print("Spawning {nb} workers")
        for _ii in range(0, nb):
            self.pool.append(self._new())

    def _new(self):
        parent, child = socket.socketpair()  # type: ignore
        pid = os.fork()
        if pid:
            child.close()
            return parent
        else:
            parent.close()
            self.childProcess(child)

    def pop(self):
        if psutil.virtual_memory().free > FREE_MEM_FOR_MODEL:
            return self._new()
        else:
            if len(self.pool) == 0:
                self.busy.acquire()
            return self.pool.pop()

    def give_back(self, element):
        self.pool.append(element)
        self.busy.release()

    def childProcess(self, child):
        r = Recognizer()
        while True:
            try:
                recvfilesize = child.recv(2048)
                child.sendall(recvfilesize)
                filesize = int(recvfilesize.decode("utf-8"))
                print(filesize)
                file = child.recv(filesize)
                print("got the file")
                file = BytesIO(file)
                with AudioFile(file) as source:
                    audio_data = r.listen(source)
                finalData = r.recognize_vosk(audio_data)
                child.sendall(finalData.encode("utf-8"))
            except Exception as e:
                child.sendall(f"exception: {e}")


pool = Pool()

my_sla_id = os.environ.get("ID", "dev")
resource = {
    "telemetry.sdk.language": "python",
    "service.name": my_sla_id,
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
    with tracer.start_as_current_span("speech_recognition"):
        if "file" not in request.files:
            file = request.get_data()
            file = BytesIO(file)
        else:
            rawfile = request.files["file"]
            file = BytesIO()
            rawfile.save(file)

        child = pool.pop()
        child.sendall(str(file.getbuffer().nbytes).encode("utf-8"))
        data = child.recv(1024)
        print(data)
        child.sendall(file.getvalue())
        print("file sent")

        finalData = child.recv(2048).decode("utf-8")
        pool.give_back(child)

        if finalData.startswith("exception"):
            print("Following error was observed:", finalData)
            print("Exiting the code.")
            abort(500, finalData)
        return json.loads(finalData)


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
    serve(app, host="0.0.0.0", port=5000, threads=20)
