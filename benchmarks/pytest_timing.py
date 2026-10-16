"""Time test calls, excluding collection and fixture setup/teardown."""
import json
import os
import time
from pathlib import Path
import pytest

@pytest.hookimpl(hookwrapper=True)
def pytest_runtest_call(item):
    start = time.perf_counter_ns()
    outcome = yield
    elapsed = time.perf_counter_ns() - start
    outcome.get_result()
    with Path(os.environ['NEAKASA_TEST_TIMINGS']).open('a') as stream:
        stream.write(json.dumps({'id':item.nodeid,'ns':elapsed})+'\n')
