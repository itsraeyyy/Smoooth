import paramiko

VPS_HOST = "169.58.72.243"
VPS_PORT = 22
VPS_USER = "root"
VPS_PASS = "Sally#123"

client = paramiko.SSHClient()
client.set_missing_host_key_policy(paramiko.AutoAddPolicy())
client.connect(VPS_HOST, port=VPS_PORT, username=VPS_USER, password=VPS_PASS, timeout=30)

commands = [
    "sed -i 's/--bind 127.0.0.1/--bind 0.0.0.0/g' /etc/systemd/system/smoooth.service",
    "sed -i 's/SMOOOTH_BIND_ADDR=127.0.0.1/SMOOOTH_BIND_ADDR=0.0.0.0/g' /etc/systemd/system/smoooth.service",
    "ufw allow 4040/tcp",
    "systemctl daemon-reload",
    "systemctl restart smoooth.service",
    "sleep 2",
    "ss -tlpn | grep 4040",
    "curl -I http://127.0.0.1:4040/"
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
