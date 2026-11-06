import pickle
from flask import Flask, request

app = Flask(__name__)

@app.route("/load", methods=["POST"])
def load():
    obj = pickle.loads(request.data)
    return str(obj)
