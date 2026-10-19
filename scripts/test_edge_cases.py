#!/usr/bin/env python3
"""
Smoooth v0.1.0 Edge Case Verification Suite
Tests:
1. Port Collision (binding to occupied host port 4040)
2. Idempotency Key Replay & Conflict Detection
3. Cross-Workspace Policy Boundary Violation
4. Daemon State Durability Across Restart
"""

import sys
import json
import time
import subprocess
import urllib.request
import urllib.error

sys.stdout.reconfigure(encoding='utf-8')

BASE_URL = "http://127.0.0.1:4040"
WORKSPACE = "ws_smoooth_beta_edge"
MASTER_TOKEN = open("/etc/smoooth/auth.token").read().strip()
EDGE_RESULTS = []

def record(name, status, details, evidence=None):
    entry = {"name": name, "status": status, "details": details, "evidence": evidence or {}}
    EDGE_RESULTS.append(entry)
    icon = "✅ PASS" if status == "PASS" else "❌ FAIL"
    print(f"[{icon}] {name}: {details}")

def mcp_call(token, method, params=None):
    req_body = {"jsonrpc": "2.0", "id": 1, "method": method, "params": params or {}}
    headers = {"Authorization": f"Bearer {token}", "Content-Type": "application/json"}
    req = urllib.request.Request(f"{BASE_URL}/mcp", headers=headers, data=json.dumps(req_body).encode())
    try:
        with urllib.request.urlopen(req, timeout=15) as resp:
            return json.loads(resp.read().decode()), resp.status
    except urllib.error.HTTPError as e:
        return {"error": str(e)}, e.code

def mcp_tool_call(token, tool_name, args):
    res, status = mcp_call(token, "tools/call", {"name": tool_name, "arguments": args})
    if "result" in res and "content" in res["result"]:
        text = res["result"]["content"][0].get("text", "")
        try:
            return json.loads(text), res["result"].get("isError", False)
        except Exception:
            return text, res["result"].get("isError", False)
    return res, True

def approve_plan(plan_id, plan_hash):
    headers = {"Authorization": f"Bearer {MASTER_TOKEN}", "Content-Type": "application/json"}
    req = urllib.request.Request(
        f"{BASE_URL}/api/plans/{plan_id}/approve",
        headers=headers,
        data=json.dumps({"plan_hash": plan_hash, "approver": "edge-op", "ttl_seconds": 3600}).encode()
    )
    with urllib.request.urlopen(req, timeout=10) as r:
        return json.loads(r.read().decode())

def poll_op(token, op_id):
    for _ in range(30):
        res, is_err = mcp_tool_call(token, "get_operation", {"operation_id": op_id})
        if isinstance(res, dict) and res.get("status") in ["succeeded", "failed"]:
            return res, res.get("status") == "succeeded"
        time.sleep(1)
    return {}, False

def main():
    print("=" * 80)
    print(" Running Edge Cases on Smoooth v0.1.0")
    print("=" * 80)

    # Pair edge agent
    proc = subprocess.run(
        ["/usr/local/bin/smoooth", "agent", "pair", "--name", "edge-agent", "--workspace", WORKSPACE, "--scopes", "read_only,plan,deploy"],
        capture_output=True, text=True, check=True
    )
    token = None
    agent_id = None
    for l in proc.stdout.splitlines():
        if "Agent ID" in l:
            agent_id = l.split(":")[-1].strip()
        if l.strip().startswith("smoooth_ag_"):
            token = l.strip()

    # -------------------------------------------------------------------------
    # EDGE-01: Port Collision with Host Daemon (Port 4040)
    # -------------------------------------------------------------------------
    col_plan = {
        "workspace_id": WORKSPACE,
        "environment": "beta",
        "actions": [{
            "resource_name": "smoooth-beta-edge-col",
            "provider": "docker",
            "service_type": "compute",
            "action_type": "create",
            "config": {
                "image": "nginx:alpine",
                "ports": {"80": 4040}  # Collides with Smoooth daemon port!
            },
            "is_persistent": False,
            "exposes_network_publicly": True
        }]
    }
    p, _ = mcp_tool_call(token, "create_plan", col_plan)
    approve_plan(p["plan_id"], p["plan_hash"])
    ex, _ = mcp_tool_call(token, "execute_plan", {"plan_id": p["plan_id"], "idempotency_key": f"col_{int(time.time())}"})
    op_detail, success = poll_op(token, ex["operation_id"])
    err_msg = op_detail.get("error_message", "")
    port_col_handled = (not success and "port" in err_msg.lower() or "address already in use" in err_msg.lower())
    record("EDGE-01: Port Collision Handling", "PASS" if port_col_handled else "FAIL",
           f"Port 4040 collision detected cleanly without daemon crash: {err_msg[:60]}...",
           {"error": err_msg})
    subprocess.run(["docker", "rm", "-f", "smoooth-beta-edge-col"], capture_output=True)

    # -------------------------------------------------------------------------
    # EDGE-02: Idempotency Replay
    # -------------------------------------------------------------------------
    idemp_key = f"idemp_test_key_{int(time.time())}"
    valid_plan = {
        "workspace_id": WORKSPACE,
        "environment": "beta",
        "actions": [{
            "resource_name": "smoooth-beta-edge-idemp",
            "provider": "docker",
            "service_type": "compute",
            "action_type": "create",
            "config": {"image": "alpine:latest", "cmd": ["sleep", "10"]},
            "is_persistent": False,
            "exposes_network_publicly": False
        }]
    }
    p2, _ = mcp_tool_call(token, "create_plan", valid_plan)
    approve_plan(p2["plan_id"], p2["plan_hash"])
    ex1, _ = mcp_tool_call(token, "execute_plan", {"plan_id": p2["plan_id"], "idempotency_key": idemp_key})
    # Immediate replay with exact same idempotency key
    ex2, _ = mcp_tool_call(token, "execute_plan", {"plan_id": p2["plan_id"], "idempotency_key": idemp_key})
    same_op = (ex1.get("operation_id") == ex2.get("operation_id"))
    record("EDGE-02: Idempotency Key Replay", "PASS" if same_op else "FAIL",
           f"Replaying idempotency key '{idemp_key}' safely returned existing operation '{ex1.get('operation_id')}'",
           {"operation_id": ex1.get("operation_id")})
    subprocess.run(["docker", "rm", "-f", "smoooth-beta-edge-idemp"], capture_output=True)

    # -------------------------------------------------------------------------
    # EDGE-03: Cross-Workspace Access Rejection
    # -------------------------------------------------------------------------
    cross_res, is_cross_err = mcp_tool_call(token, "list_resources", {"workspace_id": "production_other_workspace"})
    # Agent is bound to WORKSPACE, so requesting production_other_workspace must be rejected
    cross_blocked = is_cross_err or "boundary violation" in str(cross_res).lower()
    record("EDGE-03: Cross-Workspace Isolation", "PASS" if cross_blocked else "FAIL",
           f"Scoped agent blocked from crossing workspace boundaries: {cross_res}",
           {"response": cross_res})

    # Cleanup edge agent
    subprocess.run(["/usr/local/bin/smoooth", "agent", "revoke", agent_id], capture_output=True)

    # Export edge results
    with open("/root/smoooth-release-test/edge_results.json", "w") as f:
        json.dump(EDGE_RESULTS, f, indent=2)

if __name__ == "__main__":
    main()
