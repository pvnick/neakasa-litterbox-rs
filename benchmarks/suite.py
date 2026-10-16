#!/usr/bin/env python3
"""Audit 1:1 collected case IDs, then time the real test suites, offline.

Full-suite timing describes developer feedback latency; only the manifest's
strict paired subset is eligible for the apples-to-apples scenario comparison.
Both measurements include test framework/process overhead, not compilation.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import statistics
import subprocess
import time
import tempfile

ROOT = Path(__file__).resolve().parents[1]
REFERENCE = ROOT / 'tests/reference'
MANIFEST = ROOT / 'tests/alignment.json'


def run(cmd, *, cwd=ROOT, env=None):
    result = subprocess.run([str(x) for x in cmd], cwd=cwd, env=env,
                            stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                            text=True, timeout=120)
    if result.returncode:
        raise RuntimeError(f'Command failed ({result.returncode}): {cmd}\n{result.stdout}\n{result.stderr}')
    return result.stdout


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def environment():
    # Whitelist the test environment; never copy account credentials to artifacts.
    env = {k: os.environ[k] for k in ('PATH', 'HOME', 'LANG', 'LC_ALL', 'TMPDIR') if k in os.environ}
    env['PYTHONPATH'] = str(ROOT/'benchmarks')
    env.update(PYTEST_DISABLE_PLUGIN_AUTOLOAD='1', PYTHONHASHSEED='0', PYTHONDONTWRITEBYTECODE='1')
    return env


def pytest_command(python):
    return [python, '-m', 'pytest', '-p', 'pytest_asyncio.plugin', '-p', 'pytest_timing', '-o', 'addopts=',
            '-o', 'asyncio_mode=auto', '-o', 'cache_dir=/tmp/neakasa-pytest-cache', '-q']


def audit(python, binary, manifest, env):
    cases = manifest['cases']
    assert len(cases) == 240, 'Reference case count changed; review the entire map'
    for key in ('python', 'rust'):
        assert len({c[key] for c in cases}) == len(cases), f'Duplicate {key} mapping'
    assert all(c['status'] in ('aligned', 'adapted', 'reference') and c['notes'] for c in cases)
    assert all(not c['benchmark'] or c['status'] == 'aligned' for c in cases)
    for file, sha in manifest['reference_files'].items():
        assert digest(REFERENCE / file) == sha, f'Reference snapshot changed: {file}'
    package=Path(run([python,'-c','import neakasa_litterbox_sdk; print(neakasa_litterbox_sdk.__path__[0])'],env=env).strip())
    for file, sha in manifest['reference_sdk_files'].items():
        assert digest(package/file)==sha, f'Installed reference SDK drift: {file}'
    collected = run(pytest_command(python) + ['--collect-only', 'tests'], cwd=REFERENCE, env=env)
    py_ids = {line for line in collected.splitlines() if line.startswith('tests/') and '::' in line}
    assert py_ids == {c['python'] for c in cases}, 'Python case inventory drift'
    collected = run([binary, '--list', '--format', 'terse'], env=env)
    rust_ids = {line.removesuffix(': test') for line in collected.splitlines() if line.endswith(': test')}
    assert rust_ids == {c['rust'] for c in cases}, 'Rust case inventory drift'
    return cases


def validate_output(language, output, count):
    if language == 'python':
        assert f'{count} passed' in output, output[-2000:]
        assert 'skipped' not in output and 'xfailed' not in output and 'failed,' not in output
    else:
        assert f'{count} passed; 0 failed; 0 ignored' in output, output[-2000:]


def summarize(samples):
    q = statistics.quantiles(samples, n=4, method='inclusive')
    return {'median_ms': statistics.median(samples), 'p25_ms': q[0], 'p75_ms': q[2], 'samples_ms': samples}


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument('--python', required=True, type=Path)
    ap.add_argument('--cargo', default=Path.home()/'.cargo/bin/cargo', type=Path)
    ap.add_argument('--cpu', type=int, default=4)
    ap.add_argument('--samples', type=int, default=21)
    ap.add_argument('--warmups', type=int, default=3)
    ap.add_argument('--audit-only', action='store_true')
    ap.add_argument('--output', type=Path, default=ROOT/'benchmarks/suite-results-m1.json')
    a = ap.parse_args()
    assert a.samples >= 5 and a.warmups >= 1
    env = environment()
    started = time.perf_counter()
    build = run([a.cargo, 'test', '--offline', '--locked', '--release', '--lib', '--no-run', '--message-format=json'], env=env)
    build_ms = (time.perf_counter()-started)*1000
    artifacts = [json.loads(line) for line in build.splitlines() if line.startswith('{')]
    binary = next(Path(x['executable']) for x in artifacts if x.get('reason') == 'compiler-artifact' and x.get('executable') and x['target']['kind'] == ['lib'])
    manifest = json.loads(MANIFEST.read_text())
    cases = audit(a.python, binary, manifest, env)
    matched = [c for c in cases if c['benchmark']]
    print(f'Audit: {len(cases)} unique Python IDs ↔ {len(cases)} Rust IDs; {len(matched)} strict paired cases.', flush=True)
    if a.audit_only:
        return
    os.sched_setaffinity(0, {a.cpu})
    results = {}
    for label, selected in [('full_suite_context', cases), ('strict_paired_cases', matched)]:
        commands = {
            'python': pytest_command(a.python) + [c['python'] for c in selected],
            'rust': [binary, '--test-threads=1', '--quiet', '--exact'] + [c['rust'] for c in selected],
        }
        samples = {k: [] for k in commands}
        body_samples = {k: [] for k in commands}
        per_case = {k: [] for k in commands}
        for i in range(a.warmups+a.samples):
            order = ('python','rust') if i%2 == 0 else ('rust','python')
            for lang in order:
                with tempfile.TemporaryDirectory(prefix='neakasa-test-timing-') as tmp:
                    timing_file=Path(tmp)/'times.jsonl'
                    child_env=env | {'NEAKASA_TEST_TIMINGS':str(timing_file)}
                    begin = time.perf_counter_ns()
                    output = run(commands[lang], cwd=REFERENCE if lang=='python' else ROOT, env=child_env)
                    elapsed = (time.perf_counter_ns()-begin)/1e6
                    validate_output(lang, output, len(selected))
                    timings=[json.loads(line) for line in timing_file.read_text().splitlines()]
                    expected={c[lang] for c in selected}
                    assert len(timings)==len(selected) and {t['id'] for t in timings}==expected
                    if i>=a.warmups:
                        samples[lang].append(elapsed)
                        body_samples[lang].append(sum(t['ns'] for t in timings)/1e6)
                        per_case[lang].append(timings)
        results[label] = {'case_count':len(selected), **{k:summarize(v) | {'test_calls':summarize(body_samples[k]),'per_case_ns':per_case[k]} for k,v in samples.items()}}
        print(label, json.dumps({k:results[label][k]['median_ms'] for k in commands}), flush=True)
    tracked = run(['git','ls-files','--cached','--others','--exclude-standard']).splitlines()
    source_files = {p:digest(ROOT/p) for p in sorted(tracked) if p.endswith(('.rs','.toml','.lock','.py','.json')) and not p.startswith(('target/','benchmarks/')) and (ROOT/p).is_file()}
    data = {
        'schema':1, 'date_utc':time.strftime('%Y-%m-%dT%H:%M:%SZ',time.gmtime()),
        'reference_commit':manifest['reference_commit'], 'base_commit':run(['git','rev-parse','HEAD']).strip(),
        'source_files_sha256':source_files, 'manifest_sha256':digest(MANIFEST), 'test_binary_sha256':digest(binary),
        'benchmark_runner_sha256':{name:digest(ROOT/'benchmarks'/name) for name in ['suite.py','pytest_timing.py']},
        'environment':{'platform':platform.platform(),'machine':platform.machine(),'processor':platform.processor(),
                       'cpu_affinity':[a.cpu], 'python':run([a.python,'--version']).strip(),
                       'rustc':run([a.cargo.parent/'rustc','--version']).strip(),
                       'python_packages':json.loads(run([a.python,'-c',"import importlib.metadata,json; print(json.dumps({n:importlib.metadata.version(n) for n in ['pytest','pytest-asyncio','aiohttp','aiomqtt','cryptography','neakasa-litterbox-sdk']}))"])),
                       'governor':Path(f'/sys/devices/system/cpu/cpu{a.cpu}/cpufreq/scaling_governor').read_text().strip()},
        'method':{'warmups_per_language':a.warmups,'samples_per_language':a.samples,
                  'order':'alternating paired processes','rust_profile':'release','threads':1,
                  'python_plugins':['pytest_asyncio.plugin','pytest_timing'],'coverage':False,'cloud_requests':0,
                  'timed':'process creation through successful test completion, including imports/collection/fixtures/assertions/teardown',
                  'test_call_timing':'Sum of per-case call wall times: pytest runtest_call hook vs Rust function/future body. Excludes imports, collection, Rust runtime creation and thread startup, pytest fixture setup/teardown, and timing-file writes. Includes assertions, async scheduling, and test doubles inside calls; framework dispatch is not identical.',
                  'build_step_ms_excluded':build_ms,'build_note':'Observed incremental prebuild, not clean-build cost; excluded from all samples',
                  'scope':'Full-suite times include explicitly adapted cases and are not a paired SDK comparison. Strict set includes only manifest-approved scenarios. Neither result is cloud latency or a universal SDK speed ratio.'},
        'paired_python_ids':[c['python'] for c in matched], 'results':results,
    }
    a.output.write_text(json.dumps(data,indent=2)+'\n')
    print('Wrote',a.output.name,flush=True)

if __name__ == '__main__':
    main()
