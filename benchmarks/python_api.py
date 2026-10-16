#!/usr/bin/env python3
"""Read-only Neakasa cloud access using the owner's Android app session."""
import argparse
import asyncio
import base64
import dataclasses
import json
import logging
import hashlib
import hmac
import os
from pathlib import Path
import subprocess
import time
import uuid
import xml.etree.ElementTree as ET

from neakasa_litterbox_sdk import NeakasaClient, LoginResult, Device
from neakasa_litterbox_sdk._credentials import APP_KEY, APP_SECRET
from cryptography.hazmat.primitives.ciphers.aead import AESGCM
import neakasa_litterbox_sdk.aliyun.transport as aliyun_transport

STATE = Path.home() / '.local/state/neakasa-api'
SESSION = STATE / 'session.json'


def envelope(payload, *, api_version, iot_token=None, language='en-US', request_id=None):
    """Use explicit JSON objects instead of Fastjson-specific $ref pointers."""
    rid = request_id or str(uuid.uuid4())
    request = {'apiVer': api_version, 'language': language}
    if iot_token:
        request['iotToken'] = iot_token
    return rid, json.dumps({'id': rid, 'version': '1.0', 'request': request,
                            'params': payload}, separators=(',', ':')).encode()


aliyun_transport.build_envelope = envelope


class ReadOnlyClient(NeakasaClient):
    async def _login_rest(self):
        if not self._email or not self._password:
            raise RuntimeError('Saved account credentials are unavailable')
        return await super()._login_rest()

    async def _refresh_iot_session(self):
        return await super()._refresh_iot_session()


def save_session(session, base_url):
    STATE.mkdir(parents=True, exist_ok=True, mode=0o700)
    with os.fdopen(os.open(SESSION, os.O_WRONLY | os.O_CREAT | os.O_TRUNC, 0o600), 'w') as f:
        json.dump({'session': session.to_dict(), 'base_url': base_url}, f)


def import_android():
    raw = subprocess.check_output([
        'adb', '-s', '127.0.0.1:6520', 'shell', 'cat',
        '/data/data/com.jhkj.neakasa/shared_prefs/com.jhkj.neakasa_prefs.xml',
    ])
    prefs = {n.get('name'): n.text for n in ET.fromstring(raw)}
    user = json.loads(base64.b64decode(prefs['USER_OBJ']))
    session = LoginResult.from_json(user)
    base_url = prefs['CurrentBaseRegionURL'].rstrip('/')
    raw_iot = subprocess.check_output([
        'adb', '-s', '127.0.0.1:6520', 'shell', 'cat',
        '/data/data/com.jhkj.neakasa/shared_prefs/SecureDataStore.xml',
    ])
    iot_prefs = {n.get('name'): n.text for n in ET.fromstring(raw_iot)}
    key = hmac.new(APP_SECRET.encode(), ('safe-key-level-21' + APP_KEY).encode(),
                   hashlib.sha1).hexdigest()[:16].encode()
    encrypted = base64.b64decode(iot_prefs['KEY_IOT_CREDENTIAL_DATAv2'])
    creds = json.loads(AESGCM(key).decrypt(encrypted[:12], encrypted[12:], None))
    if not creds.get('iotToken') or creds.get('identity') != user['userInfo']['aliIdentityId']:
        raise RuntimeError('App is not signed in with a valid device session')
    session = session.with_iot_session(creds['iotToken'], 'us-east-1.api-iot.aliyuncs.com')
    save_session(session, base_url)
    return session, base_url


class AppRegion:
    name = 'US'
    def __init__(self, url):
        if url != 'https://usapi.neakasapet.com/api':
            raise ValueError('Unexpected app API endpoint')
        self.web_url = url
        self.login_url = url + '/login'


async def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--import-session', action='store_true')
    parser.add_argument('--login', action='store_true', help='Renew the session using protected local credentials')
    parser.add_argument('command', choices=['status', 'cats', 'history', 'properties'], default='cats', nargs='?')
    args = parser.parse_args()
    if args.import_session:
        session, base_url = import_android()
    else:
        saved = json.loads(SESSION.read_text())
        session, base_url = LoginResult.from_dict(saved['session']), saved['base_url']
    credentials_file = STATE / 'credentials.json'
    credentials = json.loads(credentials_file.read_text()) if credentials_file.exists() else {}
    async with ReadOnlyClient(email=credentials.get('email', ''), password=credentials.get('password', ''),
                              region=AppRegion(base_url), timeout=20) as client:
        # Resuming a saved session must never silently log in with empty credentials.
        client._login_result = session
        client._aliyun_host = session.iot_host
        if args.login:
            await client.login()
            save_session(client.login_result, base_url)
        device = Device.from_json(json.loads((STATE / 'device.json').read_text()))
        devices = [device]
        client._device_index[device.device_name] = device
        output = []
        for device in devices:
            row = {'device_name': device.device_name, 'product': device.product_name}
            if args.command == 'status':
                row['status'] = dataclasses.asdict(await client.get_status(device.device_name))
            elif args.command == 'cats':
                row['cats'] = [{'name': c.name, 'weight': c.weight, 'unit': c.unit} for c in await client.list_cats(device.device_name)]
            elif args.command == 'history':
                now = int(time.time())
                row['records'] = [dataclasses.asdict(r) for r in await client.get_toilet_records(device.device_name, now - 86400, now)]
            else:
                row['properties'] = await client._get_properties(device.device_name)
            output.append(row)
        save_session(client.login_result, base_url)
        print(json.dumps(output, indent=2, default=lambda v: v.name))


if __name__ == '__main__':
    logging.disable(logging.CRITICAL)
    try:
        asyncio.run(main())
    except Exception as exc:
        # Avoid printing server responses or URLs that might contain credentials.
        print(json.dumps({'error': type(exc).__name__, 'code': getattr(exc, 'code', None),
                          'hint': 'If authentication expired, retry with --login.'}))
        raise SystemExit(1)
