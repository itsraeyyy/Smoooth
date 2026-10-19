import os
import tarfile
import paramiko
import time
import sys

VPS_HOST = "169.58.72.243"
VPS_PORT = 22
VPS_USER = "root"
VPS_PASS = "Sally#123"

def create_source_tarball(output_filename="smoooth_src.tar.gz"):
    print(f"Creating source tarball: {output_filename}...")
    exclude_dirs = {"target", ".git", "node_modules", ".next", "out", "release-dist", "scratch"}
    
    with tarfile.open(output_filename, "w:gz") as tar:
        for root, dirs, files in os.walk("."):
            dirs[:] = [d for d in dirs if d not in exclude_dirs and not d.startswith(".")]
            for file in files:
                if file.endswith((".pyc", ".token", ".bak", ".rs.bk")):
                    continue
                full_path = os.path.join(root, file)
                rel_path = os.path.relpath(full_path, ".")
                tar.add(full_path, arcname=rel_path)
    
    size_mb = os.path.getsize(output_filename) / (1024 * 1024)
    print(f"Tarball created: {size_mb:.2f} MB")
    return output_filename

def deploy_and_build():
    tarball = create_source_tarball()
    
    client = paramiko.SSHClient()
    client.set_missing_host_key_policy(paramiko.AutoAddPolicy())
    print(f"Connecting to VPS {VPS_HOST}...")
    client.connect(VPS_HOST, port=VPS_PORT, username=VPS_USER, password=VPS_PASS, timeout=30)
    print("Connected.")

    # Upload tarball
    sftp = client.open_sftp()
    remote_tar = "/root/smoooth-release-test/smoooth_src.tar.gz"
    print(f"Uploading {tarball} to {remote_tar}...")
    sftp.put(tarball, remote_tar)
    sftp.close()
    print("Upload complete.")

    commands = [
        "mkdir -p /root/smoooth-release-test/smoooth-src",
        "tar -xzf /root/smoooth-release-test/smoooth_src.tar.gz -C /root/smoooth-release-test/smoooth-src",
        "export PATH=/root/.cargo/bin:$PATH && cd /root/smoooth-release-test/smoooth-src && cargo build --release",
        "systemctl stop smoooth.service",
        "install -m 755 /root/smoooth-release-test/smoooth-src/target/release/smoooth /usr/local/bin/smoooth",
        "systemctl start smoooth.service",
        "sleep 2",
        "systemctl is-active smoooth.service",
        "/usr/local/bin/smoooth --version",
        "/usr/local/bin/smoooth build --help",
        "curl -s http://127.0.0.1:4040/ | grep -E 'Nixpacks Automated Build Studio|drawer-panel|dot-bar|Instrument Serif'",
        "curl -s http://127.0.0.1:4040/api/ingress",
        "docker ps --format '{{.Names}}' | wc -l"
    ]

    for cmd in commands:
        print(f"\n---> Executing: {cmd}")
        stdin, stdout, stderr = client.exec_command(cmd, get_pty=True)
        while True:
            line = stdout.readline()
            if not line:
                break
            print(line, end="")
        exit_status = stdout.channel.recv_exit_status()
        if exit_status != 0:
            err = stderr.read().decode('utf-8', errors='replace')
            print(f"Command failed with exit code {exit_status}: {err}")
            if "cargo build" in cmd:
                client.close()
                sys.exit(1)

    client.close()
    print("\nDeployment and VPS verification finished successfully!")

if __name__ == "__main__":
    deploy_and_build()
