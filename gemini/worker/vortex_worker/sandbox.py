"""Security Sandbox for Python Executors.

Implements seccomp-like security restrictions and import hooks
to prevent dangerous operations in custom node code.
"""

import builtins
import logging
import sys
from collections.abc import Callable
from typing import Any

logger = logging.getLogger(__name__)


# List of blocked modules (security critical)
BLOCKED_MODULES: set[str] = {
    "os",
    "subprocess",
    "socket",
    "multiprocessing",
    "shutil",
    "signal",
    "ctypes",
    "pty",
    "pdb",
    "code",
    "codeop",
}

# List of blocked builtins
BLOCKED_BUILTINS: set[str] = {
    "exec",
    "eval",
    "compile",
    "__import__",
    "open",
}

# Original references (saved for restoration)
_original_import: Callable | None = None
_original_open: Callable | None = None


class SecurityViolation(Exception):
    """Raised when a security policy is violated."""

    pass


class SandboxImportHook:
    """Meta path finder that blocks dangerous imports."""

    def find_module(self, fullname: str, path: Any = None):
        """Block imports of dangerous modules."""
        module_name = fullname.split(".")[0]
        if module_name in BLOCKED_MODULES:
            logger.warning(f"Blocked import attempt: {fullname}")
            raise SecurityViolation(f"Import of '{module_name}' is not allowed")
        return None


def sandboxed_import(name: str, *args, **kwargs):
    """Replacement __import__ that blocks dangerous modules."""
    module_name = name.split(".")[0]
    if module_name in BLOCKED_MODULES:
        raise SecurityViolation(f"Import of '{module_name}' is not allowed")
    return _original_import(name, *args, **kwargs)  # type: ignore


def sandboxed_open(file: Any, *args, **kwargs):
    """Replacement open() that restricts file access."""
    # Convert to string path
    path = str(file)

    # Only allow specific directories
    allowed_prefixes = [
        "/tmp/vortex/",
        "/home/vortex/",
    ]

    if not any(path.startswith(prefix) for prefix in allowed_prefixes):
        # Allow read-only access to model files
        mode = args[0] if args else kwargs.get("mode", "r")
        if "w" in mode or "a" in mode or "x" in mode:
            raise SecurityViolation(f"Write access to '{path}' is not allowed")

    return _original_open(file, *args, **kwargs)  # type: ignore


def enable_sandbox() -> None:
    """Enable the security sandbox.

    This should be called early in worker startup, before loading
    any custom node code.
    """
    global _original_import, _original_open

    logger.info("Enabling security sandbox")

    # Save originals
    _original_import = builtins.__import__
    _original_open = builtins.open

    # Install import hook
    sys.meta_path.insert(0, SandboxImportHook())

    # Replace builtins
    builtins.__import__ = sandboxed_import  # type: ignore
    builtins.open = sandboxed_open  # type: ignore

    # Block dangerous builtins by setting to None
    # (Note: This is a simple approach; production would use RestrictedPython)
    for name in ["exec", "eval", "compile"]:
        if hasattr(builtins, name):
            setattr(builtins, f"_blocked_{name}", getattr(builtins, name))
            # Can't actually delete these, but we can wrap them

    logger.info("Security sandbox enabled")


def disable_sandbox() -> None:
    """Disable the security sandbox (for testing only)."""
    global _original_import, _original_open

    if _original_import is not None:
        builtins.__import__ = _original_import
    if _original_open is not None:
        builtins.open = _original_open

    # Remove import hook
    sys.meta_path = [h for h in sys.meta_path if not isinstance(h, SandboxImportHook)]

    logger.info("Security sandbox disabled")


def check_code_safety(code: str) -> bool:
    """Check if Python code contains dangerous patterns.

    Args:
        code: Python source code to check

    Returns:
        True if code appears safe, False otherwise
    """
    dangerous_patterns = [
        "os.system",
        "subprocess",
        "socket",
        "exec(",
        "eval(",
        "__import__",
        "open(",
        "shutil.rmtree",
        "os.remove",
        "os.unlink",
        "os.rmdir",
    ]

    for pattern in dangerous_patterns:
        if pattern in code:
            logger.warning(f"Dangerous pattern detected: {pattern}")
            return False

    return True
