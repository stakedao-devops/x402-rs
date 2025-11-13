#!/usr/bin/env bash

# Local development script - builds and runs x402-facilitator locally
# Does NOT push to ECR

set -e

echo "=== Building x402-facilitator for local development ==="

DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"
cd "$DIR/.."

# Build the image locally
echo "[INFO] Building Docker image..."
docker build -t x402-facilitator:local .

echo ""
echo "=== Build complete! ==="
echo ""
echo "To run locally:"
echo "  1. Create a .env file with your configuration"
echo "  2. Run: docker run --rm -p 8080:8080 --env-file .env x402-facilitator:local"
echo ""
echo "Or use docker-compose:"
echo "  1. Edit docker-compose.local.yml if needed"
echo "  2. Run: docker-compose -f docker-compose.local.yml up"
echo ""
