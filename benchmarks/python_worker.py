"""Offline worker using the installed upstream Python SDK, not a reimplementation."""
import dataclasses
import json
import sys
import time
from pathlib import Path

from neakasa_litterbox_sdk.auth.session_token import generate_session_token
from neakasa_litterbox_sdk.aliyun.signing import build_aliyun_headers
from neakasa_litterbox_sdk.models import DeviceStatus, StatusUpdate

fixtures = Path(__file__).resolve().parents[1] / "tests" / "fixtures"
models = json.loads((fixtures / "models.json").read_text())
crypto = json.loads((fixtures / "crypto.json").read_text())
case, iterations = sys.argv[1], int(sys.argv[2])
assert iterations > 0

if case == "session_token":
    token = "cafef00d" * 4
    def operation():
        return generate_session_token(token, "0123456789abcdef", "abcdef0123456789", now=1700000000.5)
    validation = operation()
    assert validation == crypto["session_token"]
    input_bytes = 46
elif case == "iot_signing":
    body = crypto["iot_body"].encode()
    def operation():
        return build_aliyun_headers("POST", "/test?x-ca-request-id=fixed", body,
            now_ms=1779157013115, nonce="fixed", date="Tue, 19 May 2026 02:16:53 GMT")
    validation = operation()
    assert validation == crypto["iot_headers"]
    input_bytes = len(body)
elif case == "status_json":
    body = json.dumps(models["properties"], separators=(",", ":"), ensure_ascii=False).encode()
    def operation():
        return DeviceStatus.from_response(json.loads(body))
    validation = dataclasses.asdict(operation())
    assert validation == models["status"]
    input_bytes = len(body)
elif case == "push_json":
    body = json.dumps(models["push"], separators=(",", ":"), ensure_ascii=False).encode()
    def operation():
        return StatusUpdate.from_push(json.loads(body))
    validation = dataclasses.asdict(operation())
    del validation["received_at"]
    assert validation == models["update"]
    input_bytes = len(body)
else:
    raise ValueError("unknown benchmark")

for _ in range(1000):
    operation()
start = time.perf_counter_ns()
for _ in range(iterations):
    operation()
elapsed = time.perf_counter_ns() - start
print(json.dumps(dict(case=case, iterations=iterations, elapsed_ns=elapsed,
    validation=validation, input_bytes=input_bytes)))
