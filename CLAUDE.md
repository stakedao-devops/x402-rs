# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is **x402-rs**, a Rust implementation of the x402 protocol for blockchain payments over HTTP. The repository contains:

- **Facilitator binary** (`src/main.rs`): Production HTTP server for verifying and settling x402 payments
- **Core library** (`src/lib.rs`): Protocol types, traits, and payment verification/settlement logic
- **Middleware crates**: `x402-axum` and `x402-reqwest` for integrating x402 into Rust applications
- **Examples**: Working demonstrations in `examples/` directory

The x402 protocol enables stateless, per-request payments where clients attach signed payment payloads to HTTP requests. The facilitator verifies signatures and settles payments on-chain using EIP-3009 (transferWithAuthorization) for EVM chains and similar mechanisms for Solana.

## Build and Development Commands

### Building

```bash
# Build main facilitator binary
cargo build

# Build release (optimized)
cargo build --release

# Build all workspace members (facilitator + middleware + examples)
just build-all
```

### Running Locally

```bash
# Run facilitator (requires .env configuration)
cargo run

# Run with custom env vars
HOST=0.0.0.0 PORT=8080 cargo run

# Quick local dev script
./scripts/local-dev.sh
```

### Testing and Quality

```bash
# Format all code in workspace
just fmt-all

# Run clippy on all workspace members
just clippy-all

# Run tests (when available)
cargo test
```

### Docker Operations

```bash
# Build Docker image locally
docker build -t x402-facilitator .

# Run container
docker run --env-file .env -p 8080:8080 x402-facilitator

# Build and push to AWS ECR (requires AWS SSO login)
export IMAGE="x402-facilitator"
./scripts/build-and-push.sh latest
```

**Important**: Before running `build-and-push.sh`, authenticate with AWS SSO:
```bash
aws sso login --profile <your-profile>
```

## Architecture

### Core Components

**Main Entry Point** (`src/main.rs`):
- Initializes Axum HTTP server on configurable `HOST:PORT` (default: `0.0.0.0:8080`)
- Loads environment configuration from `.env` file via `dotenvy`
- Sets up OpenTelemetry tracing for observability
- Configures CORS for cross-origin requests
- Initializes provider cache for multi-network RPC connections
- Implements graceful shutdown via `SigDown` signal handler

**HTTP Handlers** (`src/handlers.rs`):
- `GET /` - Landing page (Stake Capital branded HTML with network logos in `/static/`)
- `GET /verify` - Returns verification schema
- `POST /verify` - Verifies payment signature and requirements
- `GET /settle` - Returns settlement schema
- `POST /settle` - Executes payment on-chain via EIP-3009
- `GET /supported` - Lists supported networks and payment schemes
- `GET /health` - Health check endpoint
- **Static file serving** at `/static/` for logos and assets (uses `tower-http` ServeDir)

**Facilitator Trait** (`src/facilitator.rs`):
- Defines async interface for payment operations: `verify()`, `settle()`, `supported()`
- Implemented by `FacilitatorLocal` for actual blockchain interaction

**Chain Implementations**:
- `src/chain/evm.rs` - EVM network support (Base, Avalanche, Polygon, etc.)
- `src/chain/solana.rs` - Solana network support
- Uses Alloy for EVM interactions, Solana SDK for Solana

**Provider Cache** (`src/provider_cache.rs`):
- Maintains RPC connections per network configured via `RPC_URL_*` env vars
- Uses `DashMap` for concurrent access to network providers
- Networks are **only supported if their RPC URL is configured**

**Environment Configuration** (`src/from_env.rs`):
- Loads private keys for signing: `EVM_PRIVATE_KEY`, `SOLANA_PRIVATE_KEY`
- Configures supported networks based on available `RPC_URL_*` variables
- Only networks with configured RPC URLs will be active

**Payment Types** (`src/types.rs`):
- `PaymentPayload`, `PaymentRequirements`, `VerifyRequest`, `SettleRequest`
- JSON-serializable structures compatible with TypeScript/Go SDKs

### Network Configuration

The facilitator supports networks dynamically based on environment variables:

**Currently Deployed Networks** (StakeDAO production):
- Avalanche C-Chain: `RPC_URL_AVALANCHE`
- Solana: `RPC_URL_SOLANA`
- Polygon: `RPC_URL_POLYGON`
- Base: `RPC_URL_BASE`

**Additional Supported Networks** (if RPC URLs provided):
- Celo: `RPC_URL_CELO`
- Ethereum: `RPC_URL_ETHEREUM`
- Optimism: `RPC_URL_OPTIMISM`
- Arbitrum: `RPC_URL_ARBITRUM`

If an RPC URL is not set, that network is unavailable. Check logs for "no RPC URL configured, skipping" warnings.

### Workspace Structure

This is a Cargo workspace with members:
- `.` (root) - Facilitator binary and core library
- `crates/x402-axum` - Axum middleware for protecting routes with x402 payments
- `crates/x402-reqwest` - Reqwest wrapper for sending x402 payments
- `examples/x402-axum-example` - Server example using middleware
- `examples/x402-reqwest-example` - Client example sending payments

### Static Assets and Landing Page

The landing page (`GET /`) serves Stake Capital branding:
- **Logo**: `/static/stake-capital-logo.jpg`
- **Network logos**: `/static/{avalanche,solana,polygon,base}.png`
- **Favicon**: `/static/favicon.ico`
- **Styling**: Minimalist white background, monospace fonts (Courier New), rekt.news inspired
- **Content**: Displays 4 supported networks (Avalanche, Solana, Polygon, Base) with logos and API endpoint documentation

The Dockerfile **must** include: `COPY --from=builder /app/static /app/static` to include assets in the image.

## Deployment

### Automated CI/CD Pipeline (Recommended)

The facilitator uses an automated AWS CodePipeline that builds and pushes Docker images to ECR on every push to `main` branch.

**Pipeline Location**: `/sd-control-plane/terraform-stacks/control-plane/static-us-east-2/x402-facilitator-pipeline`

**How it works**:
1. Push changes to `main` branch of `stakedao-devops/x402-rs`
2. GitHub webhook triggers AWS CodePipeline
3. CodeBuild builds Docker image using `./Dockerfile`
4. Image is tagged with commit SHA and `latest`
5. Automatically pushed to ECR: `{ACCOUNT_ID}.dkr.ecr.us-east-2.amazonaws.com/stakedao/x402-facilitator`

**Deployment workflow**:
```bash
# Make changes (e.g., update landing page, add features)
git add .
git commit -m "feat: your changes"
git push origin main

# Pipeline automatically builds and pushes to ECR
# No manual AWS SSO login or docker build required!
```

To deploy the pipeline infrastructure (one-time setup):
```bash
cd /path/to/sd-control-plane/terraform-stacks/control-plane/static-us-east-2/x402-facilitator-pipeline
terragrunt apply
```

### Manual Build and Push (Alternative)

If you need to build manually (e.g., testing, hotfix):

```bash
# Authenticate with AWS (if using SSO)
aws sso login --profile <profile>

# Or if using exported credentials, unset AWS_PROFILE
unset AWS_PROFILE

# Build and push to ECR
export IMAGE="x402-facilitator"
./scripts/build-and-push.sh latest
```

This pushes to: `{ACCOUNT_ID}.dkr.ecr.us-east-2.amazonaws.com/stakecapital/x402-facilitator`

**ECR Repository**: The Docker images are stored in `stakecapital/x402-facilitator` ECR repository in `us-east-2`.

**Production Deployment**:
The service runs at `https://facilitator.prod.stake.capital/`

**AMI Building** (when infrastructure changes):
- AMI build script: `sd-amis/src/main/scripts/x402-facilitator-setup.sh`
- Build trigger: `sd-amis/src/main/scripts/builders/build-x402-facilitator.sh`
- Terraform config: `sd-control-plane/generic-systems/repos/prod/us-east-2/x402-facilitator-generic-system-deploy-source/`

**Updating Production** (without AMI rebuild):
SSH to the instance and run:
```bash
cd /opt/x402-facilitator
./update-production.sh
```

This pulls the latest image from ECR and restarts the container.

## Environment Variables

**Required**:
- `SIGNER_TYPE=private-key` (only type supported)
- `EVM_PRIVATE_KEY` - Hex private key for EVM chains (0x...)
- `SOLANA_PRIVATE_KEY` - Base58 private key for Solana

**Server**:
- `HOST` (default: `0.0.0.0`)
- `PORT` (default: `8080`)
- `RUST_LOG` (default: `info`)

**Network RPC URLs** (configure only networks you want to support):
- `RPC_URL_BASE`, `RPC_URL_BASE_SEPOLIA`
- `RPC_URL_AVALANCHE`, `RPC_URL_AVALANCHE_FUJI`
- `RPC_URL_POLYGON`, `RPC_URL_POLYGON_AMOY`
- `RPC_URL_CELO`, `RPC_URL_CELO_SEPOLIA`
- `RPC_URL_SOLANA`, `RPC_URL_SOLANA_DEVNET`

**OpenTelemetry** (optional):
- `OTEL_EXPORTER_OTLP_ENDPOINT`
- `OTEL_EXPORTER_OTLP_HEADERS`
- `OTEL_EXPORTER_OTLP_PROTOCOL`

## Key Implementation Details

### Payment Verification Flow
1. Client sends `POST /verify` with `PaymentPayload` + `PaymentRequirements`
2. Facilitator checks:
   - Network is supported (RPC URL configured)
   - Signature validity (EIP-712 for EVM, ed25519 for Solana)
   - Scheme matches (EIP-3009 vs Solana memo)
   - Value meets requirements
   - Timing is valid (not expired)
3. Returns `VerifyResponse` with status

### Payment Settlement Flow
1. Client sends `POST /settle` after successful verification
2. Facilitator executes on-chain transaction:
   - EVM: calls `transferWithAuthorization()` on USDC contract
   - Solana: submits signed transaction
3. Returns `SettleResponse` with transaction hash

### Error Handling
- Network errors → 500 with appropriate error response
- Invalid signatures → 200 OK with `VerifyResponse.valid = false`
- Unsupported networks → 200 OK with network error reason
- All errors logged with `tracing::warn!` including full request body

### CORS Configuration
Wide-open CORS for facilitator:
- `allow_origin`: Any
- `allow_methods`: GET, POST
- `allow_headers`: Any

This is intentional - the facilitator is a public service for payment verification.

## Common Pitfalls

1. **Missing RPC URLs**: If a network isn't working, check that its `RPC_URL_*` is set. The facilitator only supports networks with configured RPC endpoints.

2. **AWS SSO Authentication**: The ECR push script requires active AWS SSO session. Run `aws sso login` before pushing.

3. **Static Assets in Docker**: The landing page requires static files. Ensure Dockerfile has: `COPY --from=builder /app/static /app/static`

4. **Private Keys in .env**: Never commit `.env` files. Use `.env.example` as template. Production keys are in AWS Secrets Manager.

5. **Port Conflicts**: Default port 8080. If running multiple instances locally, change `PORT` env var.

6. **Workspace Commands**: Use `just build-all` for workspace-wide operations, not individual `cargo build` in subdirectories.

## Resources

- Protocol documentation: https://x402.org
- Coinbase x402 docs: https://docs.cdp.coinbase.com/x402/docs/overview
- Deployment guide: `DEPLOYMENT.md` (in this repo)
- Scripts documentation: `scripts/README.md`
