#!/usr/bin/python3

from scipy.spatial import distance
from flask import Flask, jsonify, request
from sentence_transformers import SentenceTransformer


app = Flask(__name__)
model = SentenceTransformer('sentence-transformers/all-mpnet-base-v2')


@app.route("/")
def root():
    return "Hello from the wembed_api!"


@app.route("/distance", methods=["GET"])
def get_distance():

    string_a = request.args.get("string_a")
    string_b = request.args.get("string_b")

    embedding_a, embedding_b = model.encode([string_a, string_b])
    the_distance = distance.cosine(embedding_a, embedding_b)

    return jsonify(
        {
            "distance": the_distance
        }
    ), 200


if __name__ == "__main__":
    app.run(debug=True, host="0.0.0.0")
