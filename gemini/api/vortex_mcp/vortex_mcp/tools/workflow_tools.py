"""
VORTEX MCP Workflow Tools

Exposes VORTEX workflow functionality to AI agents via MCP
"""
import httpx
from django.conf import settings
from django_mcp import mcp_app as mcp
from mcp.server.fastmcp import Context


@mcp.tool()
async def create_workflow(
    template_id: str,
    name: str,
    parameters: dict,
    ctx: Context
) -> dict:
    """
    Create a new workflow from template
    
    Args:
        template_id: Template identifier (e.g., "text-to-image", "video-generation")
        name: Workflow name
        parameters: Template-specific parameters
    
    Returns:
        dict with workflow_id and status
    """
    await ctx.info(f"Creating workflow '{name}' from template '{template_id}'")
    
    async with httpx.AsyncClient() as client:
        response = await client.post(
            f"{settings.VORTEX_CORE_API_URL}/api/workflows",
            json={
                "template_id": template_id,
                "name": name,
                "parameters": parameters
            }
        )
        response.raise_for_status()
        result = response.json()
    
    await ctx.info(f"Workflow created: {result.get('workflow_id')}")
    return result


@mcp.tool()
async def execute_workflow(
    workflow_id: str,
    parameters: dict,
    ctx: Context
) -> dict:
    """
    Execute a workflow with parameters
    
    Args:
        workflow_id: Workflow ID to execute
        parameters: Execution parameters (prompt, steps, cfg, etc.)
    
    Returns:
        dict with execution_id and status
    """
    await ctx.info(f"Executing workflow {workflow_id}")
    
    async with httpx.AsyncClient() as client:
        response = await client.post(
            f"{settings.VORTEX_CORE_API_URL}/api/workflows/{workflow_id}/execute",
            json=parameters
        )
        response.raise_for_status()
        result = response.json()
    
    execution_id = result.get('execution_id')
    await ctx.info(f"Execution started: {execution_id}")
    return result


@mcp.tool()
async def get_execution_status(
    execution_id: str,
    ctx: Context
) -> dict:
    """
    Get execution status and progress
    
    Args:
        execution_id: Execution ID to check
    
    Returns:
        dict with status, progress, and artifacts
    """
    async with httpx.AsyncClient() as client:
        response = await client.get(
            f"{settings.VORTEX_CORE_API_URL}/api/executions/{execution_id}"
        )
        response.raise_for_status()
        result = response.json()
    
    status = result.get('status')
    progress = result.get('progress', 0)
    
    await ctx.info(f"Execution {execution_id}: {status} ({progress}%)")
    
    if status == "COMPLETED":
        await ctx.info("Execution completed successfully")
    elif status == "FAILED":
        await ctx.error(f"Execution failed: {result.get('error')}")
    
    return result


@mcp.tool()
async def list_templates(
    category: str = None,
    ctx: Context = None
) -> dict:
    """
    List available workflow templates
    
    Args:
        category: Optional category filter (image, video, audio, text)
    
    Returns:
        dict with list of templates
    """
    url = f"{settings.VORTEX_CORE_API_URL}/api/templates"
    if category:
        url += f"?category={category}"
    
    async with httpx.AsyncClient() as client:
        response = await client.get(url)
        response.raise_for_status()
        result = response.json()
    
    if ctx:
        await ctx.info(f"Found {len(result.get('templates', []))} templates")
    
    return result


@mcp.tool()
async def approve_gate(
    approval_id: str,
    decision: str,
    reason: str = None,
    ctx: Context = None
) -> dict:
    """
    Approve or reject a human-in-the-loop gate
    
    Args:
        approval_id: Approval gate ID
        decision: "approve" or "reject"
        reason: Optional reason for decision
    
    Returns:
        dict with approval status
    """
    if decision not in ["approve", "reject"]:
        raise ValueError("decision must be 'approve' or 'reject'")
    
    if ctx:
        await ctx.info(f"Processing approval {approval_id}: {decision}")
    
    async with httpx.AsyncClient() as client:
        response = await client.post(
            f"{settings.VORTEX_CORE_API_URL}/api/approvals/{approval_id}",
            json={
                "decision": decision,
                "reason": reason
            }
        )
        response.raise_for_status()
        result = response.json()
    
    if ctx:
        await ctx.info(f"Approval {decision}d")
    
    return result


@mcp.tool()
async def cancel_execution(
    execution_id: str,
    ctx: Context
) -> dict:
    """
    Cancel a running execution
    
    Args:
        execution_id: Execution ID to cancel
    
    Returns:
        dict with cancellation status
    """
    await ctx.info(f"Cancelling execution {execution_id}")
    
    async with httpx.AsyncClient() as client:
        response = await client.post(
            f"{settings.VORTEX_CORE_API_URL}/api/executions/{execution_id}/cancel"
        )
        response.raise_for_status()
        result = response.json()
    
    await ctx.info("Execution cancelled")
    return result


@mcp.tool()
async def get_artifact(
    execution_id: str,
    artifact_type: str = "image",
    ctx: Context = None
) -> dict:
    """
    Get artifact URL from execution
    
    Args:
        execution_id: Execution ID
        artifact_type: Type of artifact (image, video, audio, text)
    
    Returns:
        dict with artifact URL and metadata
    """
    async with httpx.AsyncClient() as client:
        response = await client.get(
            f"{settings.VORTEX_CORE_API_URL}/api/executions/{execution_id}/artifacts",
            params={"type": artifact_type}
        )
        response.raise_for_status()
        result = response.json()
    
    if ctx:
        await ctx.info(f"Retrieved {artifact_type} artifact")
    
    return result
