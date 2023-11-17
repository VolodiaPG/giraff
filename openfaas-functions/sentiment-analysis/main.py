from flask import Flask, request  # type: ignore
from textblob import TextBlob  # type: ignore
from waitress import serve  # type: ignore

app = Flask(__name__)


@app.route("/", methods=["POST"])
def handle():
    blob = TextBlob(request.get_data(as_text=True))
    res = {"polarity": 0, "subjectivity": 0}

    for sentence in blob.sentences:
        res["subjectivity"] = res["subjectivity"] + sentence.sentiment.subjectivity
        res["polarity"] = res["polarity"] + sentence.sentiment.polarity

    total = len(blob.sentences)

    res["sentence_count"] = total
    res["polarity"] = res["polarity"] / total
    res["subjectivity"] = res["subjectivity"] / total
    return res


if __name__ == "__main__":
    serve(app, host="0.0.0.0", port=5000)
