from flask import Flask, abort, request  # type: ignore
from waitress import serve  # type: ignore

import speech_recognition as sr  # type: ignore

app = Flask(__name__)


@app.route("/", methods=["POST"])
def handle():
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

    return finalData


if __name__ == "__main__":
    serve(app, host="0.0.0.0", port=5000)
