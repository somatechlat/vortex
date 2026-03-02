# AWS Deployment & "Cats Flying in the Sky" Video Project
## Complete Guide: Deploy GPU Infrastructure + Generate Creative Assets

**Date:** 2026-02-21  
**Objective:** Deploy VORTEX to AWS GPU + Create "Cats Flying in Sky" video  
**Status:** READY FOR DEPLOYMENT

---

## 🚀 Phase 1: AWS Deployment

### Prerequisites
```bash
# 1. AWS CLI configured
aws configure
# OR
export AWS_ACCESS_KEY_ID=your_key
export AWS_SECRET_ACCESS_KEY=your_secret
export AWS_DEFAULT_REGION=us-east-1

# 2. Get AWS Account ID
aws sts get-caller-identity --query Account --output text

# 3. Docker running
docker ps
```

### Deployment Command
```bash
cd gemini
./scripts/deploy_aws.sh <ACCOUNT_ID> us-east-1 ml.g5.xlarge false
```

This will:
1. ✅ Build GPU worker image (docker/Dockerfile.worker.cuda)
2. ✅ Push to ECR
3. ✅ Get immutable digest (@sha256:...)
4. ✅ Apply Terraform with GPU endpoint (ml.g5.xlarge)
5. ✅ Deploy SageMaker real-time GPU endpoint

### Verify Deployment
```bash
cd infra
terraform output

# Check endpoint status
aws sagemaker describe-endpoint \
  --region us-east-1 \
  --endpoint-name vortex-gpu-test-endpoint \
  --query 'EndpointStatus'
```

Wait until status = **InService** (5-10 minutes)

---

## 🎨 Phase 2: Generate "Cats Flying in Sky" Images

### Using SageMaker GPU Endpoint Directly

**Step 1: Load Model**
```bash
cat > /tmp/load_model.json << 'EOF'
{
  "op_type": "Loader::Checkpoint",
  "params": {
    "ckpt_name": "sdxl",
    "vae": "sdxl-vae"
  },
  "inputs": {}
}
EOF

aws sagemaker-runtime invoke-endpoint \
  --region us-east-1 \
  --endpoint-name vortex-gpu-test-endpoint \
  --content-type application/json \
  --body fileb:///tmp/load_model.json \
  /tmp/load_model_out.json
```

**Step 2: Generate Cat Images**
```bash
# Create 5 variations of cats flying in the sky
for i in 1 2 3 4 5; do
  cat > /tmp/cat_$i.json << EOF
{
  "op_type": "Sampler::KSampler",
  "params": {
    "prompt": "A majestic cat with wings flying through fluffy clouds at sunset, cinematic lighting, photorealistic, 8k resolution, detailed fur, dramatic sky background",
    "negative_prompt": "blurry, low quality, artifacts, deformed, bad anatomy, disfigured",
    "steps": 50,
    "cfg": 7.5,
    "seed": $((42 + i))
  },
  "inputs": {}
}
EOF

aws sagemaker-runtime invoke-endpoint \
  --region us-east-1 \
  --endpoint-name vortex-gpu-test-endpoint \
  --content-type application/json \
  --body fileb:///tmp/cat_$i.json \
  /tmp/cat_${i}_out.json

echo "Generated cat image $i"
done
```

**Step 3: Decode to Images**
```bash
# Extract base64 from outputs and decode
for i in 1 2 3 4 5; do
  # Get the samples from output
  cat /tmp/cat_${i}_out.json | jq -r '.outputs.samples.data' | base64 -d > /tmp/cat_${i}.png
done
```

---

## 🎬 Phase 3: Create Video Using MCP

### MCP Workflow for Video Creation

**Using VORTEX MCP Server:**
```bash
# Start MCP server
cd api/vortex_mcp
uvicorn vortex_mcp.asgi:application --port 11190

# Use MCP Inspector or Claude Desktop to call:
```

**MCP Tool Sequence:**

1. **Create Video Workflow**
```json
{
  "tool": "create_workflow",
  "arguments": {
    "template_id": "text-to-video",
    "name": "Flying Cats Video",
    "parameters": {
      "images": ["/tmp/cat_1.png", "/tmp/cat_2.png", "/tmp/cat_3.png", "/tmp/cat_4.png", "/tmp/cat_5.png"],
      "fps": 24,
      "transition": "crossfade",
      "music": "ambient_sky"
    }
  }
}
```

2. **Execute Video Generation**
```json
{
  "tool": "execute_workflow",
  "arguments": {
    "workflow_id": "<from_step_1>",
    "parameters": {
      "output_format": "mp4",
      "resolution": "1920x1080",
      "bitrate": "10M"
    }
  }
}
```

3. **Get Result**
```json
{
  "tool": "get_artifact",
  "arguments": {
    "execution_id": "<from_step_2>",
    "artifact_type": "video"
  }
}
```

---

## 📋 Complete Deployment Checklist

### Pre-Deployment
- [ ] AWS CLI configured
- [ ] Docker running
- [ ] Account ID ready
- [ ] All Dockerfiles created ✅

### Deployment
- [ ] Run deploy_aws.sh
- [ ] Wait for endpoint InService
- [ ] Verify Terraform outputs

### Image Generation
- [ ] Load SDXL model
- [ ] Generate 5 cat images
- [ ] Save as PNG files

### Video Creation
- [ ] Start MCP server
- [ ] Create video workflow
- [ ] Execute and get output
- [ ] Download video

---

## 🔧 Troubleshooting

**Endpoint not InService:**
```bash
aws sagemaker describe-endpoint \
  --endpoint-name vortex-gpu-test-endpoint \
  --region us-east-1 \
  --query 'EndpointStatus'
```

**Check logs:**
```bash
aws logs describe-log-groups --region us-east-1 | grep vortex
aws logs tail /aws/sagemaker/Endpoints/vortex-gpu-test-endpoint --region us-east-1
```

**Delete and redeploy:**
```bash
cd infra
terraform destroy
./scripts/deploy_aws.sh <ACCOUNT_ID> us-east-1 ml.g5.xlarge false
```

---

## 📊 Expected Results

**GPU Instance:** ml.g5.xlarge (NVIDIA A10G, 24GB VRAM)

**Image Generation:**
- Time per image: ~30-60 seconds
- Resolution: 1024x1024
- Quality: SDXL high fidelity

**Video:**
- Format: MP4
- Resolution: 1920x1080
- Duration: 5 images × 1 second = 5 seconds
- Transitions: Crossfade

---

## 🎯 Success Criteria

✅ AWS deployment successful  
✅ SageMaker endpoint InService  
✅ 5 cat images generated  
✅ Video created successfully  
✅ MCP workflow executed  

**Total Time:** 15-30 minutes (mostly endpoint provisioning)

---

## 🚨 Important Notes

1. **GPU Cost:** ml.g5.xlarge costs ~$1.006/hour. Delete when done!
2. **Cold Start:** First inference takes 2-3 minutes (model loading)
3. **Memory:** 24GB VRAM can handle SDXL comfortably
4. **Cleanup:** Run `terraform destroy` when finished

---

## Next Steps

1. **Deploy now:** `./scripts/deploy_aws.sh <ACCOUNT_ID> us-east-1 ml.g5.xlarge false`
2. **Wait 5-10 minutes** for endpoint
3. **Generate cats** using the commands above
4. **Create video** via MCP
5. **Destroy resources** when done

**Let's do this! 🐱🛫☁️🎬**