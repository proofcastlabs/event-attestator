"""Event Attestator API package"""

import os
import logging
import secrets

import flask
import toml

from .views import default_view

logger = logging.getLogger(__package__)

MONGO_CONFIG = {
    "uri_str": "mongodb://localhost:27017/",
    "database": "sentinel",
    "collection": "signed_events",
}

MONGO_CONFIG_ENVS = {
    "MONGO_URI_STR": "uri_str",
    "MONGO_DB": "database",
    "MONGO_COLLECTION": "collection",
}

mongo_config = dict(MONGO_CONFIG)
if (config_file := os.environ.get("MONGO_CONFIG")) is not None:
    logger.info("loading mongo config from %s", config_file)
    with open(config_file, encoding="utf_8") as fpt:
        config_toml = toml.load(fpt)
    mongo_config.update(
        {k: v for k, v in config_toml["mongo"].items() if k in mongo_config}
    )
elif any(os.environ.get(env) for env in MONGO_CONFIG_ENVS):
    logger.info("loading mongo config from envs")
    for env, key in MONGO_CONFIG_ENVS.items():
        if (val := os.environ.get(env)) is not None:
            mongo_config[key] = val
else:
    logger.info("MONGO_CONFIG env not set, using defaults)")


api_app = flask.Flask(__package__)
api_app.config["SECRET_KEY"] = secrets.token_hex()
api_app.config["mongo"] = mongo_config
api_app.add_url_rule("/", view_func=default_view, methods=("POST",))
