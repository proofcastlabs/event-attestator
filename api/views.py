"""App views"""

import logging

from flask import current_app as app, request

from pymongo import MongoClient as Client

import requests


def get_signed_event(req_json):
    """Return signed events to the root view."""
    mongo = app.config["mongo"]
    client = Client(mongo["uri_str"])
    db = client[mongo["database"]]
    collection = db[mongo["collection"]]

    try:
        event_id = req_json.get("params", [])[0]
    except IndexError:
        return 'missing event id parameter, pass "params = [event_id]"', 400
    event = collection.find_one({"event_id": event_id})
    if event is not None:
        del event["_id"]
    else:
        event = {}
    return {"result": event}


REQUEST_TIMEOUT = 1  # second


def get_signer_details():
    """Return signer details to the root view."""
    url = app.config["rpc_uri_str"]

    def make_json(method, params=None):
        if params is None:
            params = []
        return {"jsonrpc": "2.0", "method": method, "params": params}

    try:
        resp_attestation_cert = requests.post(
            url,
            json=make_json("getAttestationCertificate"),
            timeout=REQUEST_TIMEOUT,
        )
        attestation_cert = resp_attestation_cert.json()["result"][
            "attestationCertificate"
        ]

        resp_pub_k = requests.post(
            url,
            json=make_json("getPublicKey"),
            timeout=REQUEST_TIMEOUT,
        )
        pub_k = resp_pub_k.json()["result"]["publicKey"]

        resp_attestation_sig = requests.post(
            url,
            json=make_json("getAttestationSignature", params=[pub_k]),
            timeout=REQUEST_TIMEOUT,
        )
        attestation_sig = resp_attestation_sig.json()["result"]["attestationSignature"]

        resp_addr = requests.post(
            url,
            json=make_json("getAddress"),
            timeout=REQUEST_TIMEOUT,
        )
        address = resp_addr.json()["result"]["address"]

        return {
            "result": {
                "publicKey": pub_k,
                "account": address,
                "attestationSignature": attestation_sig,
                "attestationCertificate": attestation_cert,
            }
        }
    except Exception as exc:
        logger = logging.getLogger(__name__)
        logger.exception("signer details got exception %s", exc)

        return "something went wrong", 500


def root_view():
    """Root view."""
    req_json = request.json
    if (method := req_json.get("method", "")) == "getSignedEvent":
        return get_signed_event(req_json)

    if method == "getSignerDetails":
        return get_signer_details()

    return f'bad method: "{method}"', 400
