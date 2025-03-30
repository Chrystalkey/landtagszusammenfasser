from enum import Enum
from typing import Tuple, Optional, List, Dict, Union
from pydantic import SecretStr

import datetime
import decimal
import sys
import os
from openapi_client.models import *

PRIMITIVE_TYPES = (float, bool, bytes, str, int)
NATIVE_TYPES_MAPPING = {
    "int": int,
    "long": int,  # TODO remove as only py3 is supported?
    "float": float,
    "str": str,
    "bool": bool,
    "date": datetime.date,
    "datetime": datetime.datetime,
    "decimal": decimal.Decimal,
    "object": object,
}


def sanitize_for_serialization(obj):
    global PRIMITIVE_TYPES
    global NATIVE_TYPES_MAPPING
    """Builds a JSON POST object.

        If obj is None, return None.
        If obj is SecretStr, return obj.get_secret_value()
        If obj is str, int, long, float, bool, return directly.
        If obj is datetime.datetime, datetime.date
            convert to string in iso8601 format.
        If obj is decimal.Decimal return string representation.
        If obj is list, sanitize each element in the list.
        If obj is dict, return the dict.
        If obj is OpenAPI model, return the properties dict.

        :param obj: The data to serialize.
        :return: The serialized form of data.
        """
    if obj is None:
        return None
    elif isinstance(obj, Enum):
        return obj.value
    elif isinstance(obj, SecretStr):
        return obj.get_secret_value()
    elif isinstance(obj, PRIMITIVE_TYPES):
        return obj
    elif isinstance(obj, list):
        return [sanitize_for_serialization(sub_obj) for sub_obj in obj]
    elif isinstance(obj, tuple):
        return tuple(sanitize_for_serialization(sub_obj) for sub_obj in obj)
    elif isinstance(obj, (datetime.datetime, datetime.date)):
        return obj.isoformat()
    elif isinstance(obj, decimal.Decimal):
        return str(obj)

    elif isinstance(obj, dict):
        obj_dict = obj
    else:
        # Convert model obj to dict except
        # attributes `openapi_types`, `attribute_map`
        # and attributes which value is not None.
        # Convert attribute name to json key in
        # model definition for request.
        if hasattr(obj, "to_dict") and callable(getattr(obj, "to_dict")):
            obj_dict = obj.to_dict()
        else:
            obj_dict = obj.__dict__

    return {key: sanitize_for_serialization(val) for key, val in obj_dict.items()}


if __name__ == "__main__":
    if len(sys.argv) < 3:
        print("Error: Specify <infile> <outfile>")
        sys.exit(1)
    with open(sys.argv[1], "r") as infile:
        string = infile.read()
        with open(sys.argv[2], "w") as outfile:
            outfile.write(sanitize_for_serialization(eval(string)))
