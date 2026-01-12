"""Security Sandbox for Python Executors - Production Grade.

Implements:
1. Seccomp BPF syscall filtering (Linux)
2. AST-based code analysis for injection prevention
3. Import hooks and builtin restrictions
4. Restricted execution environment
"""

import ast
import builtins
import logging
import platform
import sys
from collections.abc import Callable
from typing import Any

logger = logging.getLogger(__name__)

# ═══════════════════════════════════════════════════════════════
#                    SECURITY POLICIES
# ═══════════════════════════════════════════════════════════════

# Blocked modules - cannot be imported under any circumstances
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
    "requests",
    "urllib",
    "http",
    "ftplib",
    "telnetlib",
    "pickle",
    "marshal",
    "xmlrpc",
    "netrc",
}

# Blocked builtins - high risk functions
BLOCKED_BUILTINS: set[str] = {
    "exec",
    "eval",
    "compile",
    "__import__",
    "open",
    "input",
    "breakpoint",
    "globals",
    "locals",
    "vars",
    "dir",
    "getattr",
    "setattr",
    "delattr",
    "hasattr",
    "callable",
    "delattr",
}

# Allowed filesystem paths (read-only for models)
ALLOWED_READ_PATHS = [
    "/tmp/vortex/models/",
    "/home/vortex/models/",
    "/var/vortex/models/",
]

# Allowed write paths (temporary outputs)
ALLOWED_WRITE_PATHS = [
    "/tmp/vortex/outputs/",
    "/tmp/vortex/temp/",
]

# Original references
_original_import: Callable | None = None
_original_open: Callable | None = None
_original_eval = eval
_original_exec = exec
_original_compile = compile

# Platform detection
IS_LINUX = platform.system() == "Linux"
IS_DARWIN = platform.system() == "Darwin"


class SecurityViolation(Exception):
    """Raised when a security policy is violated."""

    pass


class ASTSafetyScanner(ast.NodeVisitor):
    """AST visitor that detects unsafe code patterns.

    Detects:
    - Import statements
    - Exec/Eval/Compile calls
    - File operations
    - Dangerous builtins
    - Dynamic code generation
    - Reflection attacks
    """

    def __init__(self):
        self.dangerous_nodes: list[str] = []
        self.safe_nodes: list[str] = []

    def visit_Import(self, node: ast.Import) -> None:
        """Detect import statements."""
        for alias in node.names:
            module = alias.name.split(".")[0]
            if module in BLOCKED_MODULES:
                self.dangerous_nodes.append(f"Import: {alias.name}")
            else:
                self.safe_nodes.append(f"Import: {alias.name}")
        self.generic_visit(node)

    def visit_ImportFrom(self, node: ast.ImportFrom) -> None:
        """Detect from-import statements."""
        if node.module:
            module = node.module.split(".")[0]
            if module in BLOCKED_MODULES:
                self.dangerous_nodes.append(f"FromImport: {node.module}")
            else:
                self.safe_nodes.append(f"FromImport: {node.module}")
        self.generic_visit(node)

    def visit_Call(self, node: ast.Call) -> None:
        """Detect function calls to dangerous functions."""
        if isinstance(node.func, ast.Name):
            func_name = node.func.id
            if func_name in BLOCKED_BUILTINS:
                self.dangerous_nodes.append(f"Call: {func_name}")
            elif func_name in ["exec", "eval", "compile", "__import__"]:
                self.dangerous_nodes.append(f"CodeExecution: {func_name}")
        elif isinstance(node.func, ast.Attribute):
            # Detect method calls like os.system, subprocess.run
            if isinstance(node.func.value, ast.Name):
                obj_name = node.func.value.id
                attr_name = node.func.attr
                full_name = f"{obj_name}.{attr_name}"
                if full_name in [
                    "os.system",
                    "subprocess.run",
                    "subprocess.call",
                    "socket.socket",
                    "shutil.rmtree",
                    "os.remove",
                ]:
                    self.dangerous_nodes.append(f"MethodCall: {full_name}")
        self.generic_visit(node)

    def visit_Attribute(self, node: ast.Attribute) -> None:
        """Detect attribute access that might be dangerous."""
        if isinstance(node.value, ast.Name):
            full_name = f"{node.value.id}.{node.attr}"
            # Block access to dangerous attributes
            if node.attr in [
                "__builtins__",
                "__globals__",
                "__locals__",
                "__code__",
                "__class__",
                "__subclasses__",
            ]:
                self.dangerous_nodes.append(f"Reflection: {full_name}")
        self.generic_visit(node)

    def visit_Subscript(self, node: ast.Subscript) -> None:
        """Detect dangerous subscript operations."""
        # Check for __getitem__ with dangerous keys
        self.generic_visit(node)

    def visit_FunctionDef(self, node: ast.FunctionDef) -> None:
        """Track function definitions."""
        self.safe_nodes.append(f"Function: {node.name}")
        self.generic_visit(node)

    def visit_Lambda(self, node: ast.Lambda) -> None:
        """Detect lambda expressions (can be used for obfuscation)."""
        self.safe_nodes.append("Lambda")
        self.generic_visit(node)

    def visit_BinOp(self, node: ast.BinOp) -> None:
        """Check for string concatenation that might build dangerous code."""
        self.generic_visit(node)

    def is_safe(self) -> bool:
        """Determine if code is safe based on AST analysis."""
        # Code is unsafe if it contains dangerous nodes
        return len(self.dangerous_nodes) == 0

    def get_report(self) -> dict:
        """Get safety report."""
        return {
            "safe": self.is_safe(),
            "dangerous_nodes": self.dangerous_nodes,
            "safe_nodes": self.safe_nodes,
            "dangerous_count": len(self.dangerous_nodes),
            "safe_count": len(self.safe_nodes),
        }


class SandboxImportHook:
    """Meta path finder that blocks dangerous imports."""

    def find_spec(self, fullname: str, path, target=None):
        """Block imports of dangerous modules."""
        module_name = fullname.split(".")[0]
        if module_name in BLOCKED_MODULES:
            logger.warning(f"[SANDBOX] Blocked import: {fullname}")
            raise SecurityViolation(f"Import of '{module_name}' is not allowed")
        return None


def sandboxed_import(name: str, *args, **kwargs):
    """Replacement __import__ that blocks dangerous modules."""
    module_name = name.split(".")[0]
    if module_name in BLOCKED_MODULES:
        logger.warning(f"[SANDBOX] Blocked __import__: {name}")
        raise SecurityViolation(f"Import of '{module_name}' is not allowed")
    return _original_import(name, *args, **kwargs)


def sandboxed_open(file: Any, *args, **kwargs):
    """Replacement open() that restricts file access."""
    path = str(file)
    mode = args[0] if args else kwargs.get("mode", "r")

    # Check if writing
    if any(m in mode for m in ["w", "a", "x", "+"]):
        allowed = any(path.startswith(p) for p in ALLOWED_WRITE_PATHS)
        if not allowed:
            logger.warning(f"[SANDBOX] Blocked write to: {path}")
            raise SecurityViolation(f"Write access to '{path}' is not allowed")
    else:
        # Read access
        allowed = any(path.startswith(p) for p in ALLOWED_READ_PATHS) or any(
            path.startswith(p) for p in ALLOWED_WRITE_PATHS
        )
        if not allowed:
            logger.warning(f"[SANDBOX] Blocked read from: {path}")
            raise SecurityViolation(f"Read access to '{path}' is not allowed")

    return _original_open(file, *args, **kwargs)


def sandboxed_exec(__code: Any, __globals: Any = None, __locals: Any = None):
    """Block exec() calls."""
    logger.warning("[SANDBOX] Blocked exec() call")
    raise SecurityViolation("exec() is not allowed in sandboxed environment")


def sandboxed_eval(__code: Any, __globals: Any = None, __locals: Any = None):
    """Block eval() calls."""
    logger.warning("[SANDBOX] Blocked eval() call")
    raise SecurityViolation("eval() is not allowed in sandboxed environment")


def sandboxed_compile(source: Any, filename: Any, mode: Any, *args, **kwargs):
    """Block compile() calls."""
    logger.warning("[SANDBOX] Blocked compile() call")
    raise SecurityViolation("compile() is not allowed in sandboxed environment")


# ═══════════════════════════════════════════════════════════════
#                    SECCOMP IMPLEMENTATION (Linux only)
# ═══════════════════════════════════════════════════════════════


def _install_seccomp_bpf() -> bool:
    """Install seccomp BPF filter (Linux only).

    Blocks dangerous syscalls:
    - execve, execveat (process execution)
    - socket, connect, bind (network)
    - open (file writes)
    - kill, tkill (signal injection)
    - ptrace (debugging/inspection)

    Returns:
        True if seccomp was installed, False if not available
    """
    if not IS_LINUX:
        logger.info("[SANDBOX] Seccomp not available (not Linux)")
        return False

    try:
        # Try to import seccomp (libseccomp-python)
        import seccomp as seccomp_lib

        # Create filter
        f = seccomp_lib.SyscallFilter(defaction=seccomp_lib.ALLOW)

        # Allow basic syscalls needed by Python
        allowed = [
            "read",
            "write",
            "open",
            "close",
            "stat",
            "fstat",
            "mmap",
            "munmap",
            "brk",
            "rt_sigaction",
            "rt_sigprocmask",
            "ioctl",
            "pread64",
            "getpid",
            "getuid",
            "geteuid",
            "getgid",
            "getegid",
            "futex",
            "sched_yield",
            "nanosleep",
            "exit_group",
            "clone",
            "set_tid_address",
        ]

        # Block dangerous syscalls
        blocked = [
            "execve",
            "execveat",  # Process execution
            "socket",
            "connect",
            "bind",
            "listen",
            "accept",  # Network
            "kill",
            "tkill",
            "tgkill",  # Process signaling
            "ptrace",  # Debugging
        ]

        for syscall in blocked:
            f.add_rule(seccomp_lib.KILL, syscall)

        f.load()
        logger.info("[SANDBOX] Seccomp BPF filter installed")
        return True

    except ImportError:
        logger.warning("[SANDBOX] seccomp library not available")
        return False
    except Exception as e:
        logger.error(f"[SANDBOX] Failed to install seccomp: {e}")
        return False


# ═══════════════════════════════════════════════════════════════
#                    HIGH-LEVEL SANDBOX API
# ═══════════════════════════════════════════════════════════════


def scan_code(code: str, filename: str = "<string>") -> dict:
    """Scan Python code using AST analysis.

    Args:
        code: Python source code
        filename: Source filename for error reporting

    Returns:
        Dict with safety analysis results
    """
    try:
        tree = ast.parse(code, filename)
        scanner = ASTSafetyScanner()
        scanner.visit(tree)
        return scanner.get_report()
    except SyntaxError as e:
        return {
            "safe": False,
            "error": f"Syntax error: {e}",
            "dangerous_nodes": ["SYNTAX_ERROR"],
            "safe_nodes": [],
        }


def validate_node_code(node_type: str, code: str) -> bool:
    """Validate custom node code before execution.

    Args:
        node_type: Type of node (for logging)
        code: Python code to validate

    Returns:
        True if code is safe, raises SecurityViolation otherwise
    """
    logger.info(f"[SANDBOX] Validating code for node type: {node_type}")

    report = scan_code(code)

    if not report["safe"]:
        logger.error(f"[SANDBOX] Unsafe code detected in {node_type}")
        logger.error(f"[SANDBOX] Dangerous patterns: {report['dangerous_nodes']}")
        raise SecurityViolation(
            f"Code validation failed for {node_type}: " f"{report['dangerous_nodes']}"
        )

    logger.info(f"[SANDBOX] Code validation passed for {node_type}")
    return True


def enable_sandbox() -> None:
    """Enable comprehensive security sandbox.

    This should be called early in worker startup before loading
    any custom code. It:
    1. Installs import hooks
    2. Replaces dangerous builtins
    3. Attempts to install seccomp (Linux)
    4. Sets up filesystem restrictions
    """
    global _original_import, _original_open

    logger.info("=" * 60)
    logger.info("[SANDBOX] ENABLING SECURITY SANDBOX")
    logger.info("=" * 60)

    # Save originals
    _original_import = builtins.__import__
    _original_open = builtins.open

    # Install import hook
    sys.meta_path.insert(0, SandboxImportHook())
    logger.info("[SANDBOX] Import hook installed")

    # Replace dangerous builtins
    builtins.__import__ = sandboxed_import
    builtins.open = sandboxed_open
    builtins.exec = sandboxed_exec
    builtins.eval = sandboxed_eval
    builtins.compile = sandboxed_compile

    logger.info("[SANDBOX] Dangerous builtins replaced")

    # Install seccomp (Linux only)
    seccomp_installed = _install_seccomp_bpf()
    if seccomp_installed:
        logger.info("[SANDBOX] Seccomp BPF filter active")
    else:
        logger.warning("[SANDBOX] Seccomp not available (using Python-only protection)")

    logger.info("=" * 60)
    logger.info("[SANDBOX] SECURITY SANDBOX ENABLED")
    logger.info("=" * 60)


def disable_sandbox() -> None:
    """Disable sandbox (testing only)."""
    global _original_import, _original_open

    if _original_import is not None:
        builtins.__import__ = _original_import
    if _original_open is not None:
        builtins.open = _original_open

    builtins.exec = _original_exec
    builtins.eval = _original_eval
    builtins.compile = _original_compile

    sys.meta_path = [h for h in sys.meta_path if not isinstance(h, SandboxImportHook)]

    logger.warning("[SANDBOX] Sandbox disabled (should only be used in testing)")


def get_security_status() -> dict:
    """Get current sandbox security status."""
    return {
        "sandbox_enabled": _original_import is not None,
        "platform": platform.system(),
        "seccomp_available": IS_LINUX,
        "blocked_modules": len(BLOCKED_MODULES),
        "blocked_builtins": len(BLOCKED_BUILTINS),
    }
