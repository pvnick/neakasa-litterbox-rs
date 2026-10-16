import json,re
from pathlib import Path
m=json.loads(Path('tests/alignment.json').read_text());pure=Path('tests/aligned/pure.rs').read_text();out=['#![allow(non_snake_case)]\nmod pure;\nmod support;\nmod structural;\nmod client_cases;\nmod transport_cases;\nmod stream_cases;\nmod timing;']
for group in sorted({c['rust'].split('::')[1] for c in m['cases']}):
 out.append(f'mod {group} {{')
 for c in m['cases']:
  if c['rust'].split('::')[1]!=group:continue
  n=c['python'].split('::')[1];fn=c['rust'].split('::')[2];base=n.split('[')[0]
  ispure=f'"{base}"' in pure
  if ispure or group in ['region','packaging_config'] or base.startswith('test_expect_object') or base in ['test_user_agent_tracks_installed_version','test_sdk_version_falls_back_when_package_metadata_is_missing']:
   out.append(f'#[test]\nfn {fn}() {{super::timing::sync({json.dumps(c["rust"])}, || super::pure::run({json.dumps(group)},{json.dumps(n)}));}}')
  else:
   handler='client_cases' if group.startswith('client_') or group in ['aliyun_handshake','aliyun_session_refresh','eu_gateway_integration'] else 'transport_cases' if group in ['http_transport','aliyun_transport'] else 'stream_cases'
   out.append(f'#[tokio::test]\nasync fn {fn}() {{super::timing::asynchronous({json.dumps(c["rust"])}, super::{handler}::run({json.dumps(group)},{json.dumps(n)})).await;}}')
 out.append('}')
Path('tests/aligned/mod.rs').write_text('\n'.join(out)+'\n')
