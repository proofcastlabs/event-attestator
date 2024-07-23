"""App views"""

from flask import current_app as app, request

from pymongo import MongoClient as Client


def default_view():
    """Default view."""
    req_json = request.json
    if (method := req_json.get("method", "")) == "getSignedEvent":
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
    return f'bad method: "{method}"', 400
