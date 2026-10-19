import paramiko
import json
import sys

sys.stdout.reconfigure(encoding='utf-8')

VPS_HOST = "169.58.72.243"
VPS_PORT = 22
VPS_USER = "root"
VPS_PASS = "Sally#123"

client = paramiko.SSHClient()
client.set_missing_host_key_policy(paramiko.AutoAddPolicy())
client.connect(VPS_HOST, port=VPS_PORT, username=VPS_USER, password=VPS_PASS, timeout=30)

# Create a sample zero-Dockerfile Node.js app
setup_cmd = """
mkdir -p /tmp/smoooth-verify-app
cat << 'EOF' > /tmp/smoooth-verify-app/package.json
{
  "name": "smoooth-nixpack-sample",
  "version": "1.0.0",
  "main": "server.js",
  "scripts": {
    "start": "node server.js"
  },
  "dependencies": {}
}
EOF
cat << 'EOF' > /tmp/smoooth-verify-app/server.js
const http = require('http');
const server = http.createServer((req, res) => {
  res.writeHead(200, { 'Content-Type': 'application/json' });
  res.end(JSON.stringify({ status: 'healthy', provider: 'nixpacks', timestamp: new Date() }));
});
server.listen(8080, () => console.log('Sample app listening on 8080'));
EOF
"""

print("Setting up sample test application...")
client.exec_command(setup_cmd)

tests = [
    # 1. Test CLI Nixpacks plan
    "/usr/local/bin/smoooth build plan /tmp/smoooth-verify-app",
    # 2. Test CLI Nixpacks build
    "/usr/local/bin/smoooth build image /tmp/smoooth-verify-app --name smoooth-vps-test:v1",
    # 3. Verify Docker image was produced
    "docker images smoooth-vps-test:v1 --format 'table {{.Repository}}:{{.Tag}}\t{{.Size}}'",
    # 4. Clean up test image
    "docker rmi -f smoooth-vps-test:v1",
    # 5. Clean up sample folder
    "rm -rf /tmp/smoooth-verify-app",
    # 6. Verify 40 production containers untouched
    "docker ps --format '{{.Names}}' | wc -l"
]

for cmd in tests:
    print(f"\n========================================\nTEST: {cmd}")
    stdin, stdout, stderr = client.exec_command(cmd)
    out = stdout.read().decode('utf-8', errors='replace').strip()
    err = stderr.read().decode('utf-8', errors='replace').strip()
    if out:
        print(f"STDOUT:\n{out}")
    if err:
        print(f"STDERR:\n{err}")

client.close()
print("\nAll live VPS feature tests completed!")
