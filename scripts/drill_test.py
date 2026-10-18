import subprocess
import urllib.request
import json
import time
import socket

print("======================================================================")
print(" Smoooth Beta Release Drill: End-to-End Test")
print("======================================================================")

# Step 6: Agent Pairing & MCP Connection
print("\n--- Step 6: Agent Pairing & MCP Connection ---")
pair_proc = subprocess.run(
    ['/usr/local/bin/smoooth', 'agent', 'pair', '--name', 'claude-code-beta', '--workspace', 'beta-drill', '--scopes', 'read_only,plan,deploy'],
    capture_output=True, text=True, check=True
)

agent_token = None
agent_id = None
for line in pair_proc.stdout.splitlines():
    line_s = line.strip()
    if 'Agent ID' in line_s:
        agent_id = line_s.split(':')[-1].strip()
    if line_s.startswith('smoooth_ag_') or 'SMOOOTH_AUTH_TOKEN=' in line_s:
        if line_s.startswith('smoooth_ag_'):
            agent_token = line_s
        continue
    print(line)

assert agent_token is not None, "Failed to extract agent token!"
print(f"  🔑 Bearer Token Issued : {agent_token[:14]}...[REDACTED_FOR_SECURITY]")

# Test MCP tools/list
req = urllib.request.Request(
    'http://127.0.0.1:4040/mcp',
    headers={'Authorization': f'Bearer {agent_token}', 'Content-Type': 'application/json'},
    data=json.dumps({'jsonrpc': '2.0', 'id': 1, 'method': 'tools/list', 'params': {}}).encode()
)
res = json.loads(urllib.request.urlopen(req).read().decode())
tools = res['result']['tools']
print(f"MCP tools/list succeeded! Found {len(tools)} available tools:")
for t in tools:
    print(f"  • {t['name']}")

# Step 7 & 8: Deploy Application & Stateful Redis Database with Persistence
print("\n--- Step 7 & 8: Deploy Real Workloads & Verify Database Persistence ---")
plan_payload = {
    'workspace_id': 'beta-drill',
    'environment': 'beta',
    'actions': [
        {
            'resource_name': 'drill_redis_db',
            'provider': 'redis',
            'service_type': 'database',
            'action_type': 'create',
            'config': {
                'port': 16379,
                'password': 'drill_redis_secure_pass',
                'persist': True
            },
            'is_persistent': True,
            'exposes_network_publicly': False
        },
        {
            'resource_name': 'drill_web_app',
            'provider': 'docker',
            'service_type': 'compute',
            'action_type': 'create',
            'config': {
                'image': 'nginx:alpine',
                'ports': {
                    '80': 18080
                }
            },
            'is_persistent': False,
            'exposes_network_publicly': True
        }
    ]
}

create_plan_req = urllib.request.Request(
    'http://127.0.0.1:4040/mcp',
    headers={'Authorization': f'Bearer {agent_token}', 'Content-Type': 'application/json'},
    data=json.dumps({
        'jsonrpc': '2.0',
        'id': 2,
        'method': 'tools/call',
        'params': {
            'name': 'create_plan',
            'arguments': plan_payload
        }
    }).encode()
)
plan_res = json.loads(urllib.request.urlopen(create_plan_req).read().decode())
plan_data = json.loads(plan_res['result']['content'][0]['text'])
print(f"Plan created successfully: ID={plan_data['plan_id']}, Hash={plan_data['plan_hash']}")

# Approve Plan via Operator (Master Token) per governance policy
master_token = open('/etc/smoooth/auth.token').read().strip()
approve_req = urllib.request.Request(
    f"http://127.0.0.1:4040/api/plans/{plan_data['plan_id']}/approve",
    headers={'Authorization': f'Bearer {master_token}', 'Content-Type': 'application/json'},
    data=json.dumps({
        'plan_hash': plan_data['plan_hash'],
        'approver': 'beta-operator@local',
        'ttl_seconds': 3600
    }).encode()
)
approve_res = json.loads(urllib.request.urlopen(approve_req).read().decode())
print(f"Plan approved by operator: {approve_res}")

# Execute Plan
exec_req = urllib.request.Request(
    'http://127.0.0.1:4040/mcp',
    headers={'Authorization': f'Bearer {agent_token}', 'Content-Type': 'application/json'},
    data=json.dumps({
        'jsonrpc': '2.0',
        'id': 4,
        'method': 'tools/call',
        'params': {
            'name': 'execute_plan',
            'arguments': {
                'plan_id': plan_data['plan_id'],
                'idempotency_key': f"drill_exec_{int(time.time())}"
            }
        }
    }).encode()
)
exec_res = json.loads(urllib.request.urlopen(exec_req).read().decode())
exec_text = exec_res['result']['content'][0]['text']
print(f"Plan execution initiated: {exec_text}")
exec_info = json.loads(exec_text)
op_id = exec_info['operation_id']

# Poll get_operation until terminal state
print(f"Polling operation '{op_id}' until completion...")
for attempt in range(40):
    time.sleep(1)
    get_op_req = urllib.request.Request(
        'http://127.0.0.1:4040/mcp',
        headers={'Authorization': f'Bearer {agent_token}', 'Content-Type': 'application/json'},
        data=json.dumps({
            'jsonrpc': '2.0',
            'id': 50 + attempt,
            'method': 'tools/call',
            'params': {
                'name': 'get_operation',
                'arguments': {'operation_id': op_id}
            }
        }).encode()
    )
    get_op_res = json.loads(urllib.request.urlopen(get_op_req).read().decode())
    op_detail = json.loads(get_op_res['result']['content'][0]['text'])
    status = op_detail.get('status')
    progress = op_detail.get('progress_percent', 0)
    print(f"  [Attempt {attempt+1}] Status: {status}, Progress: {progress}%")
    if status == 'succeeded':
        evidence = op_detail.get('evidence') or []
        print(f"Plan execution SUCCEEDED! Evidence: {len(evidence)} verified health proofs recorded in audit trail:")
        for idx, ev in enumerate(evidence, 1):
            print(f"  Proof {idx}: {ev.get('message')} (latency: {ev.get('latency_ms')}ms)")
        assert len(evidence) >= 2, f"Expected at least 2 health proofs, got {len(evidence)}"
        break
    elif status == 'failed':
        raise RuntimeError(f"Plan execution FAILED: {op_detail.get('error_message')}")
else:
    raise TimeoutError(f"Operation {op_id} did not finish within 40 seconds")

# Verify Web App
time.sleep(1)
app_req = urllib.request.Request('http://127.0.0.1:18080')
app_html = urllib.request.urlopen(app_req).read().decode()
assert "Welcome to nginx" in app_html, "Nginx web app verification failed!"
print("Web app HTTP verification: SUCCESS (nginx running on http://127.0.0.1:18080)")

# Verify Redis writes
s = socket.socket()
s.connect(('127.0.0.1', 16379))
s.sendall(b"AUTH drill_redis_secure_pass\r\n")
auth_ack = s.recv(1024)
assert b"+OK" in auth_ack, f"Redis auth failed: {auth_ack}"
s.sendall(b"SET beta_drill_record \"verified_persistence_val_99\"\r\n")
set_ack = s.recv(1024)
assert b"+OK" in set_ack, f"Redis set failed: {set_ack}"
s.close()
print("Redis write verification: SUCCESS (Record written: beta_drill_record)")

# Restart Redis container and verify persistence
print("Restarting Redis container to test persistence across container restart...")
subprocess.run(['docker', 'restart', 'drill_redis_db'], check=True)
time.sleep(2)

s2 = socket.socket()
s2.connect(('127.0.0.1', 16379))
s2.sendall(b"AUTH drill_redis_secure_pass\r\n")
s2.recv(1024)
s2.sendall(b"GET beta_drill_record\r\n")
get_ack = s2.recv(1024)
s2.close()
assert b"verified_persistence_val_99" in get_ack, f"Persistence verification failed! Value: {get_ack}"
print(f"Redis persistence verification: SUCCESS (Retrieved: {get_ack.decode().strip()})")

# Step 9: Dashboard Inspection from Separate Session
print("\n--- Step 9: Dashboard & Operation History Verification ---")
master_token = open('/etc/smoooth/auth.token').read().strip()

# 1. Unauthenticated request to /api/operations must fail
unauth_failed = False
try:
    urllib.request.urlopen('http://127.0.0.1:4040/api/operations')
except urllib.error.HTTPError as e:
    assert e.code == 401
    unauth_failed = True
assert unauth_failed, "Unauthenticated request was NOT rejected!"
print("Unauthenticated access rejection: SUCCESS (401 Unauthorized)")

# 2. Authenticated request to /api/operations
ops_req = urllib.request.Request(
    'http://127.0.0.1:4040/api/operations',
    headers={'Authorization': f'Bearer {master_token}'}
)
ops_res = json.loads(urllib.request.urlopen(ops_req).read().decode())
operations = ops_res.get('operations', [])
print(f"Dashboard operation history: SUCCESS ({len(operations)} persistent operations recorded in state)")
if operations:
    latest_op = operations[0]
    print(f"  Latest Op ID: {latest_op.get('id')}")
    print(f"  Status: {latest_op.get('status')}")
    print(f"  Workspace: {latest_op.get('workspace_id')}")

# Step 10: Failure Reporting and Self-Healing Recovery
print("\n--- Step 10: Failure Reporting & Recovery Drill ---")
# Simulate container stoppage
print("Simulating container crash/stop on drill_web_app...")
subprocess.run(['docker', 'stop', 'drill_web_app'], check=True)

# Call reconcile_resource via MCP
reconcile_req = urllib.request.Request(
    'http://127.0.0.1:4040/mcp',
    headers={'Authorization': f'Bearer {agent_token}', 'Content-Type': 'application/json'},
    data=json.dumps({
        'jsonrpc': '2.0',
        'id': 5,
        'method': 'tools/call',
        'params': {
            'name': 'reconcile_resource',
            'arguments': {
                'resource_id': 'drill_web_app'
            }
        }
    }).encode()
)
rec_res = json.loads(urllib.request.urlopen(reconcile_req).read().decode())
print(f"reconcile_resource response: {rec_res['result']['content'][0]['text']}")

# Confirm recovery
time.sleep(2)
app_req2 = urllib.request.Request('http://127.0.0.1:18080')
app_html2 = urllib.request.urlopen(app_req2).read().decode()
assert "Welcome to nginx" in app_html2, "Self-healing reconciliation failed!"
print("Self-healing recovery: SUCCESS (Web app revived and healthy on 18080)")

# Step 11: Revocation & Credential Hygiene
print("\n--- Step 11: Immediate Credential Revocation & Hygiene ---")
print(f"Revoking test agent '{agent_id}' to prevent credential leakage...")
subprocess.run(['/usr/local/bin/smoooth', 'agent', 'revoke', agent_id], check=True)

# Verify that the revoked token is immediately rejected
revoked_rejected = False
try:
    rev_req = urllib.request.Request(
        'http://127.0.0.1:4040/mcp',
        headers={'Authorization': f'Bearer {agent_token}', 'Content-Type': 'application/json'},
        data=json.dumps({'jsonrpc': '2.0', 'id': 99, 'method': 'tools/list', 'params': {}}).encode()
    )
    urllib.request.urlopen(rev_req)
except urllib.error.HTTPError as e:
    assert e.code == 401
    revoked_rejected = True
assert revoked_rejected, "Revoked token was NOT rejected!"
print("Post-drill revocation verification: SUCCESS (Revoked token immediately rejected with 401 Unauthorized)")

# Clean up test containers
subprocess.run(['docker', 'rm', '-f', 'drill_redis_db', 'drill_web_app'], capture_output=True)
print("Drill test workloads cleaned up.")

print("\n======================================================================")
print(" Steps 6 through 11 Release Drill: ALL PASSED")
print("======================================================================")
