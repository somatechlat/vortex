import logging
from typing import Any, Dict

logger = logging.getLogger(__name__)

# ═══════════════════════════════════════════════════════════════
#                    MESSAGE REGISTRY
# ═══════════════════════════════════════════════════════════════

MESSAGES: Dict[str, str] = {
    # MCP Operations
    "MCP_TOOL_REGISTERED": "McpService: Registered tool {tool_name}",
    "MCP_EXECUTION_SUCCESS": "McpService: Executing tool {tool_name}",
    "MCP_TOOL_NOT_FOUND": "McpService: Tool {tool_name} not found",

    # Authentication
    "AUTH_TOKEN_INVALID": "Invalid bearer token for {resource}",
    "AUTH_TOKEN_MISSING": "No API token configured for {resource}",

    # Generic Errors
    "ERROR_GENERIC": "An unexpected error occurred: {error}",
    "ERROR_NOT_FOUND": "Resource not found: {resource_id}",
    "ERROR_UNAUTHORIZED": "Unauthorized access to {resource}",
}

def get_message(code: str, **kwargs: Any) -> str:
    """
    Retrieve a formatted message by its code.
    Follows SOMA Rule 11 for centralized user-facing text.
    """
    template = MESSAGES.get(code)
    if not template:
        logger.warning(f"Message code '{code}' not found in registry")
        return code

    try:
        return template.format(**kwargs)
    except KeyError as e:
        logger.error(f"Missing parameter '{e}' for message code '{code}'")
        return template
    except Exception as e:
        logger.error(f"Error formatting message '{code}': {e}")
        return template
