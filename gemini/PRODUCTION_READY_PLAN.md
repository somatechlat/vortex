# VORTEX Production Deployment Plan
## Complete Implementation & Testing Strategy

**Status:** Ready for Implementation  
**Timeline:** 4 weeks to production  
**MCP:** Django-based using django-mcp package ✅

---

## Phase 1: MCP Server (COMPLETE ✅)

### Implementation
- ✅ Django 5 + django-mcp package
- ✅ 7 MCP tools exposed to AI agents
- ✅ Proper ASGI configuration
- ✅ Health check endpoint
- ✅ Connection to VORTEX Core API

### MCP Tools Available
1. `create_workflow` - Create from template
2. `execute_workflow` - Run with parameters
3. `get_execution_status` - Check progress
4. `list_templates` - Browse templates
5. `approve_gate` - HITL approval
6. `cancel_execution` - Cancel running
7. `get_artifact` - Get output URLs

### Run MCP Server
```bash
cd api/vortex_mcp
pip install -r requirements.txt
uvicorn vortex_mcp.asgi:application --host 0.0.0.0 --port 11190
```

### Test with MCP Inspector
```bash
python manage.py mcp_inspector http://localhost:11190/mcp/sse
```

---

## Phase 2: Core Fixes (Week 1)

### Critical Issues

**1. Remove `.unwrap()` Calls (49+ instances)**
Location: `crates/vortex-core/src/*.rs`

Strategy:
- Replace with `?` operator
- Add proper error context
- Implement recovery strategies

**2. Update Documentation**
- Change "Svelte 5" → "Lit 3.x" in all SRS docs
- Update architecture diagrams
- Create ADR-003 for framework decision

**3. Complete Missing Tasks**
- T-03: Worker heartbeat + timeout
- T-04: JWT middleware
- T-05: Level-parallel execution
- T-06: VRAM arbiter integration
- T-07: Cancellation tokens

---

## Phase 3: Docker Infrastructure (Week 2)

### Services Required

**Local Development Stack:**
```yaml
services:
  postgres:       # Port 5432
  spicedb:        # Port 8080, 50051
  keycloak:       # Port 8081
  vortex-core:    # Port 11188, 11189
  vortex-worker:  # IPC via UDS
  vortex-ui:      # Port 11173
  vortex-mcp:     # Port 11190 (NEW)
```

**MCP Server Addition:**
```yaml
vortex-mcp:
  build: ./api/vortex_mcp
  ports: ["11190:11190"]
  environment:
    VORTEX_CORE_API_URL: http://vortex-core:11188
    POSTGRES_HOST: postgres
  depends_on: [vortex-core, postgres]
```

---

## Phase 4: Testing Strategy (Week 3)

### Test Pyramid

**1. Unit Tests**
```bash
# Rust
cargo test --workspace

# Python
pytest worker/tests/

# MCP
pytest api/vortex_mcp/tests/
```

**2. Integration Tests**
- Rust ↔ Python IPC
- Shared memory round-trip
- MCP ↔ Core API
- SpiceDB authorization

**3. MCP Agent Tests**
Test scenarios using MCP Inspector or Claude Desktop:

**Scenario 1: Simple Workflow**
```json
{
  "tool": "create_workflow",
  "arguments": {
    "template_id": "text-to-image",
    "name": "Test Workflow",
    "parameters": {
      "model": "sdxl",
      "steps": 30
    }
  }
}
```

**Scenario 2: Execute & Monitor**
```json
{
  "tool": "execute_workflow",
  "arguments": {
    "workflow_id": "wf-123",
    "parameters": {
      "prompt": "cyberpunk city",
      "cfg": 7.5
    }
  }
}
```

**Scenario 3: HITL Approval**
```json
{
  "tool": "approve_gate",
  "arguments": {
    "approval_id": "appr-456",
    "decision": "approve",
    "reason": "Reviewed and approved"
  }
}
```

**4. Load Tests**
- 100 concurrent workflows
- 1000 nodes in single graph
- Memory leak detection
- VRAM pressure testing

---

## Phase 5: AWS Deployment (Week 4)

### Infrastructure Components

**Terraform Modules:**
1. VPC + Networking
2. ECS Fargate (vortex-core)
3. RDS PostgreSQL (Multi-AZ)
4. ElastiCache Redis
5. SageMaker Serverless Endpoints
6. S3 + CloudFront
7. CloudWatch Logs/Metrics

**Deployment Steps:**
```bash
cd infra
terraform init
terraform plan
terraform apply
```

---

## MCP Agent Test Plan

### Test 1: Create & Execute Workflow

**Agent Actions:**
1. List available templates
2. Create workflow from template
3. Execute workflow with parameters
4. Monitor execution status
5. Retrieve artifact URL

**Expected Result:**
- Workflow created successfully
- Execution completes
- Artifact URL returned
- Image accessible

**Validation:**
```bash
# Check execution in database
psql -d vortex -c "SELECT * FROM runs WHERE id='run-123';"

# Verify artifact in S3
aws s3 ls s3://vortex-artifacts/run-123/
```

### Test 2: HITL Approval Flow

**Agent Actions:**
1. Create workflow requiring approval
2. Execute workflow
3. Receive approval request
4. Approve gate
5. Execution continues

**Expected Result:**
- Execution pauses at gate
- Agent receives approval metadata
- After approval, execution resumes
- Audit log records decision

**Validation:**
```bash
# Check approval in database
psql -d vortex -c "SELECT * FROM approvals WHERE id='appr-456';"

# Verify audit trail
psql -d vortex -c "SELECT * FROM audit_log WHERE approval_id='appr-456';"
```

### Test 3: Cancellation

**Agent Actions:**
1. Start long-running workflow
2. Cancel mid-execution
3. Verify cancellation

**Expected Result:**
- Execution stops within 1 node boundary
- Status = CANCELLED
- Resources cleaned up
- No orphaned processes

**Validation:**
```bash
# Check execution status
psql -d vortex -c "SELECT status FROM runs WHERE id='run-789';"

# Verify no orphaned workers
ps aux | grep vortex-worker
```

### Test 4: Error Handling

**Agent Actions:**
1. Execute workflow with invalid parameters
2. Verify error response
3. Check error details

**Expected Result:**
- Proper error message
- Error code (VE-XXX)
- Helpful context

---

## Success Criteria

### Production Ready Checklist

**Code Quality:**
- [ ] Zero `.unwrap()` in production code
- [ ] `cargo clippy -D warnings` passes
- [ ] `ruff check` passes
- [ ] `mypy --strict` passes
- [ ] All tests passing (>90% coverage)

**Documentation:**
- [ ] All "Svelte 5" changed to "Lit 3.x"
- [ ] ADR-003 created
- [ ] API documentation complete
- [ ] Deployment runbook complete

**Infrastructure:**
- [ ] Docker Compose working locally
- [ ] Terraform deploys to AWS
- [ ] All services healthy
- [ ] Monitoring/alerting configured

**MCP Integration:**
- [ ] MCP server running on port 11190
- [ ] All 7 tools working
- [ ] MCP Inspector connects
- [ ] Agent can execute workflows

**Testing:**
- [ ] Unit tests pass
- [ ] Integration tests pass
- [ ] MCP agent tests pass
- [ ] Load tests pass (100 concurrent)

**Security:**
- [ ] Secrets in Vault (not env vars)
- [ ] SpiceDB authorization working
- [ ] JWT middleware active
- [ ] HITL approval gates enforced

**Performance:**
- [ ] API p95 < 100ms
- [ ] Execution p95 < 5s
- [ ] VRAM arbiter prevents OOM
- [ ] Cancellation < 1 node boundary

---

## Next Steps

### Week 1: Core Fixes
1. Fix all `.unwrap()` calls
2. Update documentation
3. Complete T-03, T-04

### Week 2: Infrastructure
1. Create Docker Compose
2. Add MCP service
3. Test locally

### Week 3: Testing
1. Write MCP agent tests
2. Run integration tests
3. Load testing

### Week 4: Deployment
1. Deploy to AWS
2. Run production tests
3. Monitor & optimize

---

## MCP Server Details

### Endpoints

**MCP Protocol:**
- `POST /mcp/` - Main MCP endpoint
- `GET /mcp/sse` - Server-Sent Events
- `POST /mcp/messages/` - Message posting

**Health:**
- `GET /health` - Health check

### Configuration

**Environment Variables:**
```bash
VORTEX_CORE_API_URL=http://localhost:11188
POSTGRES_HOST=localhost
POSTGRES_DB=vortex
POSTGRES_USER=vortex
POSTGRES_PASSWORD=***
VORTEX_MCP_SECRET_KEY=***
```

### Port Assignment (VIBE Port Authority)
- 11188: VORTEX Core HTTP
- 11189: VORTEX Core WebSocket
- 11190: VORTEX MCP Server ✅
- 11191: Prometheus Metrics
- 11173: UI Dev Server

---

## Conclusion

The MCP server is now properly implemented using django-mcp package per VIBE rules. The production deployment plan is complete and ready for execution.

**Key Achievements:**
✅ MCP server using Django + django-mcp
✅ 7 tools exposed to AI agents
✅ Proper ASGI configuration
✅ Integration with VORTEX Core API
✅ Complete testing strategy
✅ Production deployment plan

**Ready for:** Implementation and testing with real MCP clients (Claude Desktop, MCP Inspector, custom agents)
