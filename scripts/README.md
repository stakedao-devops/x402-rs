# X402 Facilitator Build Scripts

Scripts for building and deploying x402-facilitator to AWS ECR.

## Available Scripts

### `build-and-push.sh`

Builds the Docker image and pushes it to AWS ECR.

**Usage:**
```bash
./build-and-push.sh [TAG]
```

**Examples:**
```bash
# Push with version tag
./build-and-push.sh v1.0.0

# Push as latest
./build-and-push.sh latest

# Default (latest)
./build-and-push.sh
```

**What it does:**
1. Sources `docker-init.sh` to set up ECR connection
2. Builds Docker image from parent directory
3. Tags image with provided tag and `latest`
4. Pushes both tags to ECR

**ECR Repository:** `{ACCOUNT_ID}.dkr.ecr.us-east-2.amazonaws.com/stakedao/x402-facilitator`

---

### `docker-init.sh`

Initializes Docker and ECR environment. Called by other scripts.

**What it does:**
1. Gets AWS account ID and region
2. Creates ECR repository if it doesn't exist
3. Logs into ECR
4. Sets up environment variables for other scripts

**Environment Variables Set:**
- `ACCOUNT_ID`: AWS account identifier
- `AWS_REGION`: Target AWS region (default: us-east-2)
- `ecr_repository`: Full ECR repository URI

---

### `update-production.sh`

Updates the x402-facilitator service in production by pulling latest image from ECR.

**Usage:**
Run this on the production EC2 instance:
```bash
ssh ubuntu@facilitator.prod.stake.capital
cd /opt/x402-facilitator
./update-production.sh
```

**What it does:**
1. Logs into ECR
2. Pulls latest image from ECR
3. Stops current container
4. Starts new container with latest image
5. Verifies service is running

**No AMI rebuild required!**

---

## Prerequisites

- Docker installed
- AWS CLI configured with proper credentials
- `jq` installed
- Access to AWS ECR repository

## ECR Repository Structure

```
stakedao/x402-facilitator
├── latest          # Always newest build
├── v1.0.0          # Semantic version tags
├── v1.0.1
├── main            # Built from main branch
└── ...
```

## Development Workflow

1. **Make changes** to x402-rs code
2. **Build and push** to ECR:
   ```bash
   ./scripts/build-and-push.sh v1.0.1
   ```
3. **Update production** (no AMI rebuild):
   ```bash
   ssh ubuntu@facilitator.prod.stake.capital
   cd /opt/x402-facilitator
   ./update-production.sh
   ```

## Troubleshooting

### "Repository does not exist"
The script automatically creates the repository. If it fails:
```bash
aws ecr create-repository --repository-name stakedao/x402-facilitator --region us-east-2
```

### "Cannot connect to ECR"
Re-authenticate:
```bash
aws ecr get-login-password --region us-east-2 | docker login --username AWS --password-stdin $(aws sts get-caller-identity | jq -r .Account).dkr.ecr.us-east-2.amazonaws.com
```

### "Build failed"
Check Docker is running and you have sufficient disk space:
```bash
docker info
df -h
```

## See Also

- [DEPLOYMENT.md](../DEPLOYMENT.md) - Complete deployment guide
- [AMI Build Process](../../../../sd-amis/src/main/scripts/x402-facilitator-setup.sh) - AMI build script
