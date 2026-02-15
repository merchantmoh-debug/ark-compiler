"""
Ark Security Module — Sandbox enforcement and capability checks.

Extracted from ark.py (Phase 72: Structural Hardening).
"""
import os
import sys
import socket
import ipaddress
import urllib.parse
import urllib.request


class SandboxViolation(Exception):
    pass


class LinearityViolation(Exception):
    pass


# ─── Capability-Token System ─────────────────────────────────────────────────
#
# Replaces the binary ALLOW_DANGEROUS_LOCAL_EXECUTION env var with granular caps.
# Usage: ARK_CAPABILITIES="exec,net,fs_write,fs_read,thread,ai"
#
# Backward compat: ALLOW_DANGEROUS_LOCAL_EXECUTION=true grants ALL capabilities.

def _load_capabilities():
    """Load capabilities from environment."""
    # Backward compatibility: old env var grants everything
    if os.environ.get("ALLOW_DANGEROUS_LOCAL_EXECUTION", "false").lower() == "true":
        return {"exec", "net", "fs_write", "fs_read", "thread", "ai", "all"}
    
    raw = os.environ.get("ARK_CAPABILITIES", "")
    if not raw:
        return set()
    return set(cap.strip() for cap in raw.split(",") if cap.strip())


CAPABILITIES = _load_capabilities()


def has_capability(cap: str) -> bool:
    """Check if a capability is granted."""
    return "all" in CAPABILITIES or cap in CAPABILITIES


def check_capability(cap: str):
    """Require a capability, raising SandboxViolation if not granted."""
    if not has_capability(cap):
        raise SandboxViolation(
            f"Capability '{cap}' not granted. "
            f"Set ARK_CAPABILITIES={cap} or ALLOW_DANGEROUS_LOCAL_EXECUTION=true to enable."
        )


# ─── Path Security ───────────────────────────────────────────────────────────

def check_path_security(path, is_write=False):
    if has_capability("all"):
        return

    # Path Traversal Check
    abs_path = os.path.realpath(path)
    cwd = os.getcwd()

    # Check if path is within CWD (or is CWD itself)
    if os.path.commonpath([cwd, abs_path]) != cwd:
        raise SandboxViolation(f"Access outside working directory is forbidden: {path} (Resolved to: {abs_path})")

    if is_write:
        # Require fs_write capability
        check_capability("fs_write")
        
        # Protect system files from being overwritten in sandbox mode
        meta_dir = os.path.dirname(os.path.realpath(__file__))
        repo_root = os.path.dirname(meta_dir)

        # Protected directories
        protected_dirs = [
            "meta", "core", "lib", "src", "tests",
            "apps", "benchmarks", "docs", "examples", "ops", "web",
            ".git", ".agent", ".antigravity", ".context", "artifacts"
        ]
        for d in protected_dirs:
            protected_path = os.path.realpath(os.path.join(repo_root, d))
            if abs_path.startswith(protected_path):
                raise SandboxViolation(f"Writing to protected directory is forbidden: {d}")

        # Protected root files
        protected_files = [
            "Cargo.toml", "README.md", "LICENSE", "requirements.txt",
            "MANUAL.md", "ARK_OMEGA_POINT.md", "SWARM_PLAN.md", "CLA.md",
            "Dockerfile", "docker-compose.yml", "sovereign_launch.bat",
            "pyproject.toml", "Cargo.lock", "debug_build.py"
        ]
        for f in protected_files:
            protected_file_path = os.path.realpath(os.path.join(repo_root, f))
            if abs_path == protected_file_path:
                raise SandboxViolation(f"Writing to protected file is forbidden: {f}")


def check_exec_security():
    """Check if exec capability is granted."""
    check_capability("exec")


# ─── URL Security ────────────────────────────────────────────────────────────

def validate_url_security(url):
    try:
        parsed = urllib.parse.urlparse(url)
    except Exception as e:
        raise Exception(f"Invalid URL: {e}")

    if parsed.scheme not in ('http', 'https'):
        raise Exception(f"URL scheme '{parsed.scheme}' is not allowed (only http/https)")

    hostname = parsed.hostname
    if not hostname:
        raise Exception("Invalid URL: missing hostname")

    # Resolve hostname to IP
    try:
        addr_info = socket.getaddrinfo(hostname, None)
    except socket.gaierror as e:
        raise Exception(f"DNS resolution failed for {hostname}: {e}")

    for _, _, _, _, sockaddr in addr_info:
        ip_str = sockaddr[0]
        try:
            ip = ipaddress.ip_address(ip_str)
        except ValueError:
            continue

        if ip.is_loopback:
            continue  # Allow localhost for local testing/dev

        if ip.is_private or ip.is_link_local or ip.is_multicast or ip.is_reserved:
            raise SandboxViolation(f"Access to private/local/reserved IP '{ip_str}' is forbidden")

        if str(ip) == "0.0.0.0":
            raise SandboxViolation("Access to 0.0.0.0 is forbidden")


class SafeRedirectHandler(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, req, fp, code, msg, headers, newurl):
        validate_url_security(newurl)
        return super().redirect_request(req, fp, code, msg, headers, newurl)
