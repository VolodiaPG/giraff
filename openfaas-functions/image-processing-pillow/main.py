from io import BytesIO

from flask import Flask, abort, request, send_file  # type: ignore
from PIL import Image  # type: ignore
from waitress import serve  # type: ignore

app = Flask(__name__)


def serve_pil_image(pil_img):
    img_io = BytesIO()
    pil_img.save(img_io, "JPEG", quality=70)
    img_io.seek(0)
    return send_file(img_io, mimetype="image/jpeg")


@app.route("/", methods=["POST"])
def handle():
    if "file" not in request.files:
        abort(400)

    file = request.files["file"]

    im = Image.open(file)
    resized_im = im.resize((400, 400))
    return serve_pil_image(resized_im)


if __name__ == "__main__":
    serve(app, host="0.0.0.0", port=5000)
