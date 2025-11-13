#!/bin/bash

set -e
set -x

SUPPORTED_AWS_REGIONS=${SUPPORTED_AWS_REGIONS:-"us-east-2"}

if [ -z "$IMAGE" ]; then
  echo "No IMAGE provided"
  exit 1
fi

declare DIR
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"
cd "$DIR/.." || exit

declare IMAGE_NAME="stakedao/${IMAGE}"
echo "[INFO] Getting account ID..."
declare ACCOUNT_ID
ACCOUNT_ID=$(aws sts get-caller-identity | jq -r .Account)

echo "[INFO] Account ID = $ACCOUNT_ID"
echo "[INFO] Getting region..."
echo "[INFO] Region = ${AWS_REGION:=$(aws configure get region)}"

declare ecr_repository="${ACCOUNT_ID}.dkr.ecr.${AWS_REGION}.amazonaws.com/${IMAGE_NAME}"

IFS=',' read -ra REGIONS <<< "${SUPPORTED_AWS_REGIONS[$i]}"
for REGION in "${REGIONS[@]}"; do
  REPO="${ACCOUNT_ID}.dkr.ecr.${REGION}.amazonaws.com/${IMAGE_NAME}"

  if ! aws ecr describe-repositories --region "${REGION}" | jq -r '.repositories[].repositoryUri' | grep -q "^${REPO}\$" ; then
    echo "[INFO] Creating ECR repository: ${IMAGE_NAME} in ${REGION}"
    if ! aws ecr create-repository --repository-name "${IMAGE_NAME}" --region "${REGION}"; then
       echo "[ERROR] Failed to create ECR repository!"
       exit 1
    fi
  else
    echo "[INFO] ECR repository already exists: ${IMAGE_NAME} in ${REGION}"
  fi
done

declare CREATED
if date --utc --iso-8601=seconds >/dev/null 2>&1; then
  # GNU coreutils' version of date
  CREATED=$(date --utc --iso-8601=seconds)
elif gdate --utc --iso-8601=seconds >/dev/null 2>&1; then
  # We're probably on macOS with GNU coreutils installed via homebrew
  CREATED=$(gdate --utc --iso-8601=seconds)
else
  # We're probably on macOS
  CREATED=$(date -u +"%Y-%m-%dT%H:%M:%S+00:00")
fi

declare REVISION
set +e
REVISION=$(git rev-parse HEAD || echo "unknown")
set -e

echo "[INFO] ISO Date = ${CREATED}"
echo "[INFO] Revision = ${REVISION}"

# Login to ECR
echo "[INFO] Logging into ECR..."
aws ecr get-login-password --region "${AWS_REGION}" | docker login --username AWS --password-stdin "${ACCOUNT_ID}.dkr.ecr.${AWS_REGION}.amazonaws.com"
