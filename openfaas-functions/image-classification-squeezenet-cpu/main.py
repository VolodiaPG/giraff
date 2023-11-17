import os

import torch  # type: ignore
from flask import Flask, abort, request  # type: ignore
from PIL import Image  # type: ignore
from torchvision import models, transforms  # type: ignore
from waitress import serve  # type: ignore

app = Flask(__name__)

model_path = os.environ["SQUEEZENET_MODEL"]


@app.route("/", methods=["POST"])
def handle():
    if "file" not in request.files:
        abort(400)

    file = request.files["file"]

    model = models.squeezenet1_1()  # initialize the model
    model.load_state_dict(torch.load(model_path))

    input_image = Image.open(file)
    preprocess = transforms.Compose(
        [
            transforms.Resize(256),
            transforms.CenterCrop(224),
            transforms.ToTensor(),
            transforms.Normalize(mean=[0.485, 0.456, 0.406], std=[0.229, 0.224, 0.225]),
        ]
    )
    input_tensor = preprocess(input_image)
    input_batch = input_tensor.unsqueeze(0)  # Create mini batch as expected by model

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


if __name__ == "__main__":
    serve(app, host="0.0.0.0", port=5000)
