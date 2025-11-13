# X402 Facilitator Deployment Guide

This is the StakeDAO fork of x402-rs configured to build and deploy via AWS ECR.

## Architecture Overview

```
┌─────────────────┐
│ Local Dev       │
│ (customizations)│
└────────┬────────┘
         │
         ├─ Edit code
         ├─ Build & Push to ECR
         │
         v
┌─────────────────┐
│ AWS ECR         │
│ x402-facilitator│
└────────┬────────┘
         │
         ├─ AMI pulls from ECR (initial setup)
         ├─ Production pulls updates (no AMI rebuild)
         │
         v
┌─────────────────┐
│ Production EC2  │
│ x402-facilitator│
└─────────────────┘
```

## Prerequisites

- Docker
- AWS CLI configured
- jq installed
- Access to AWS ECR (account ID configured)
- Git

## Development Workflow

### 1. Make Code Changes

Edit your customizations in this repository:
```bash
cd /Users/sjaramillo/Documents/dm/stakedao/vcs/x402-rs
# Make your changes...
git add .
git commit -m "Your changes"
git push origin main
```

### 2. Build and Push to ECR

```bash
cd /Users/sjaramillo/Documents/dm/stakedao/vcs/x402-rs

# Build and push with version tag
./scripts/build-and-push.sh v1.0.0

# Or build and push as latest
./scripts/build-and-push.sh latest
```

This script will:
- Create ECR repository if it doesn't exist
- Build the Docker image from source
- Tag it with your version and `latest`
- Push both tags to ECR

### 3. Update Production (No AMI Rebuild Required!)

**Option A: SSH into production instance**
```bash
ssh ubuntu@facilitator.prod.stake.capital
cd /opt/x402-facilitator
# Copy the update script
./update-production.sh
```

**Option B: Use the update script from x402-rs repo**
```bash
# Copy update script to production
scp scripts/update-production.sh ubuntu@facilitator.prod.stake.capital:/opt/x402-facilitator/

# SSH and run
ssh ubuntu@facilitator.prod.stake.capital
cd /opt/x402-facilitator
./update-production.sh
```

## AMI Build Process

The AMI build process (only needed for initial setup or infrastructure changes):

### Located at:
`/Users/sjaramillo/Documents/dm/stakedao/vcs/sd-amis/src/main/scripts/x402-facilitator-setup.sh`

### What it does:
1. Installs Docker, Rust, and dependencies
2. Clones this fork: `https://github.com/stakedao-devops/x402-rs`
3. Builds the image and pushes to ECR
4. Creates `.env` template with placeholders
5. Creates `docker-compose.yml` configured to pull from ECR

### Build AMI:
```bash
cd /Users/sjaramillo/Documents/dm/stakedao/vcs/sd-amis/src/main/scripts/builders
./build-x402-facilitator.sh
```

### Environment Variables:
- `X402_VERSION`: Git branch/tag to use (default: `main`)
- `AWS_REGION`: AWS region for ECR (default: `us-east-2`)

## ECR Repository

**Repository Name:** `stakedao/x402-facilitator`

**Full URI:** `{ACCOUNT_ID}.dkr.ecr.us-east-2.amazonaws.com/stakedao/x402-facilitator`

**Tags:**
- `latest`: Always points to the most recent build
- `v*`: Semantic version tags (e.g., `v1.0.0`, `v1.0.1`)
- `main`: Built from main branch

## Production Configuration

### Location
All files located at: `/opt/x402-facilitator/`

### Files
```
/opt/x402-facilitator/
├── .env                    # Environment variables (secrets)
├── docker-compose.yml      # Docker Compose configuration
├── x402-rs/               # Source code (for reference)
└── update-production.sh   # Update helper script
```

### Environment Variables (.env)
```bash
# Server Configuration
HOST=0.0.0.0
PORT=80
RUST_LOG=info

# Signer Configuration (REQUIRED)
SIGNER_TYPE=private-key
EVM_PRIVATE_KEY=0x...
SOLANA_PRIVATE_KEY=...

# RPC URLs - Mainnet Networks
RPC_URL_BASE=https://...
RPC_URL_AVALANCHE=https://...
RPC_URL_POLYGON=https://...
RPC_URL_CELO=https://...
RPC_URL_SOLANA=https://...
```

These are populated automatically from AWS Secrets Manager during boot via the userdata script.

## Userdata Script

Located at: `/Users/sjaramillo/Documents/dm/stakedao/vcs/sd-terraform-modules/validator-system/validator/templates/prod/userdata-x402-facilitator.tpl`

This script:
1. Retrieves secrets from AWS Secrets Manager
2. Populates `.env` file with actual values
3. Runs `docker-compose up -d` to start the facilitator
4. Adds monitoring and graceful shutdown scripts

## Production Commands

### View Logs
```bash
cd /opt/x402-facilitator
docker-compose logs -f x402-facilitator
```

### Restart Service
```bash
cd /opt/x402-facilitator
docker-compose restart x402-facilitator
```

### Stop Service
```bash
cd /opt/x402-facilitator
docker-compose down
```

### Start Service
```bash
cd /opt/x402-facilitator
docker-compose up -d
```

### Check Status
```bash
docker ps | grep x402-facilitator
curl http://localhost:80/
curl https://facilitator.prod.stake.capital/
```

### Update to Latest Image (from ECR)
```bash
cd /opt/x402-facilitator
./update-production.sh
```

## Troubleshooting

### Image not found in ECR
```bash
# Check if image exists
aws ecr describe-images --repository-name stakedao/x402-facilitator --region us-east-2

# If missing, rebuild and push
cd /Users/sjaramillo/Documents/dm/stakedao/vcs/x402-rs
./scripts/build-and-push.sh latest
```

### Cannot pull from ECR
```bash
# Re-login to ECR
aws ecr get-login-password --region us-east-2 | docker login --username AWS --password-stdin $(aws sts get-caller-identity | jq -r .Account).dkr.ecr.us-east-2.amazonaws.com
```

### Service fails to start
```bash
# Check logs
docker-compose logs x402-facilitator

# Check .env configuration
cat /opt/x402-facilitator/.env

# Verify secrets from AWS Secrets Manager
aws secretsmanager get-secret-value --secret-id x402-facilitator-us-east-2.additional --region us-east-2
```

## Benefits of This Approach

1. **No AMI Rebuilds**: Update code without rebuilding entire AMI
2. **Fast Deployments**: `docker pull` + `docker-compose up` takes seconds
3. **Easy Rollbacks**: Just pull previous tag from ECR
4. **Version Control**: All images tagged and stored in ECR
5. **Consistent Builds**: Same image runs locally and in production
6. **Separation of Concerns**: Infrastructure (AMI) vs Application (Docker image)

## Version History

Track your deployments by tagging images:

```bash
# Build specific version
./scripts/build-and-push.sh v1.0.0

# Update production to specific version
# (modify docker-compose.yml image tag, then restart)
```

## References

- **AMI Build Scripts**: `/Users/sjaramillo/Documents/dm/stakedao/vcs/sd-amis/src/main/scripts/`
- **Terraform Config**: `/Users/sjaramillo/Documents/dm/stakedao/vcs/sd-control-plane/generic-systems/repos/prod/us-east-2/x402-facilitator-generic-system-deploy-source/`
- **Userdata Template**: `/Users/sjaramillo/Documents/dm/stakedao/vcs/sd-terraform-modules/validator-system/validator/templates/prod/userdata-x402-facilitator.tpl`
- **Original Repo**: `https://github.com/x402-rs/x402-rs`
- **StakeDAO Fork**: `https://github.com/stakedao-devops/x402-rs`
