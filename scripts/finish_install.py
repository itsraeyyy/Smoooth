import paramiko

VPS_HOST = "169.58.72.243"
VPS_PORT = 22
VPS_USER = "root"
VPS_PASS = "Sally#123"

client = paramiko.SSHClient()
client.set_missing_host_key_policy(paramiko.AutoAddPolicy())
client.connect(VPS_HOST, port=VPS_PORT, username=VPS_USER, password=VPS_PASS, timeout=30)

commands = [
    "systemctl stop smoooth.service",
    "install -m 755 /root/smoooth-release-test/smoooth-src/target/release/smoooth /usr/local/bin/smoooth",
    "systemctl start smoooth.service",
    "sleep 2",
    "systemctl is-active smoooth.service",
    "/usr/local/bin/smoooth --version",
    "/usr/local/bin/smoooth build --help",
    "curl -s http://127.0.0.1:4040/ | grep 'Nixpacks Automated Build Studio'",
    "curl -s http://127.0.0.1:4040/ | grep 'drawer-panel'",
    "curl -s http://127.0.0.1:4040/ | grep 'Instrument Serif'",
    "docker ps --format '{{.Names}}' | wc -l"
]

for cmd in commands:
    print(f"\n========================================\nEXEC: {cmd}")
    stdin, stdout, stderr = client.exec_command(cmd)
    out = stdout.read().decode('utf-8', errors='replace').strip()
    err = stderr.read().decode('utf-8', errors='replace').strip()
    if out:
        print(f"STDOUT:\n{out}")
    if err:
        print(f"STDERR:\n{err}")

client.close()
