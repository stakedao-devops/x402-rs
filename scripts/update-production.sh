#!/usr/bin/env bash

# Script to update x402-facilitator in production from ECR
# Run this on the production EC2 instance

set -e

X402_HOME=/opt/x402-facilitator

echo "=== Updating x402-facilitator from ECR ==="

# Get AWS Account ID and Region
ACCOUNT_ID=$(aws sts get-caller-identity | jq -r .Account)
AWS_REGION=${AWS_REGION:-us-east-2}
ECR_REPOSITORY="${ACCOUNT_ID}.dkr.ecr.${AWS_REGION}.amazonaws.com/stakecapital/x402-facilitator"

echo "ECR Repository: $ECR_REPOSITORY:latest"

# Login to ECR
echo "=== Logging into ECR ==="
aws ecr get-login-password --region "${AWS_REGION}" | docker login --username AWS --password-stdin "${ACCOUNT_ID}.dkr.ecr.${AWS_REGION}.amazonaws.com"

# Pull latest image
echo "=== Pulling latest image ==="
docker pull "${ECR_REPOSITORY}:latest"

# Stop and remove old container
echo "=== Stopping old container ==="
cd $X402_HOME
docker-compose down

# Start with new image
echo "=== Starting new container ==="
docker-compose up -d

# Wait for service to be ready
sleep 5

# Verify service is running
if docker ps | grep -q x402-facilitator; then
    echo "=== ✓ x402 facilitator updated successfully ==="
    docker-compose logs --tail 50 x402-facilitator
else
    echo "=== ✗ ERROR: x402 facilitator failed to start ==="
    docker-compose logs x402-facilitator
    exit 1
fi
