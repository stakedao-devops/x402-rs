#!/usr/bin/env bash

set -e

IMAGE_TAG=${1:-latest}
echo "[INFO] Image tag: $IMAGE_TAG"

# Set image name
IMAGE="x402-facilitator"
echo "[INFO] Image: $IMAGE"

# Source docker-init to set up ECR
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"
source "$DIR/docker-init.sh"

# Build the Docker image
echo "[INFO] Building Docker image..."
cd "$DIR/.."
docker build --compress -t "${ecr_repository}:${IMAGE_TAG}" .

# Also tag as latest
echo "[INFO] Tagging as latest..."
docker tag "${ecr_repository}:${IMAGE_TAG}" "${ecr_repository}:latest"

# Push both tags to ECR
echo "[INFO] Pushing ${IMAGE_TAG} to ECR..."
docker push "${ecr_repository}:${IMAGE_TAG}"

echo "[INFO] Pushing latest to ECR..."
docker push "${ecr_repository}:latest"

echo "[INFO] Successfully pushed to ECR:"
echo "  - ${ecr_repository}:${IMAGE_TAG}"
echo "  - ${ecr_repository}:latest"
