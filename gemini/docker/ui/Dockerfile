# ═══════════════════════════════════════════════════════════════════════════════
# VORTEX UI Dockerfile - Svelte 5 + Bun + Nginx
# ═══════════════════════════════════════════════════════════════════════════════
#
# ⚠️  DEPLOYMENT STACK: Colima + Minikube + Tilt (NO DOCKER DESKTOP)
#
# This image is designed to be deployed via:
#   1. Colima - Docker runtime (NOT Docker Desktop)
#   2. Minikube - Local Kubernetes cluster (docker driver)
#   3. Tilt - Development workflow orchestration
#
# DO NOT USE DOCKER DESKTOP OR DOCKER COMPOSE DIRECTLY.
# Always run: `tilt up` from the repository root.
#
# Build via Tilt (automatic):
#   $ cd /path/to/vortex && tilt up
#
# Manual build (for debugging only):
#   $ eval $(minikube docker-env)
#   $ docker build -t vortex-ui:latest -f docker/ui/Dockerfile ui/
#
# ═══════════════════════════════════════════════════════════════════════════════

FROM oven/bun:1 AS builder

WORKDIR /app

# Copy package files
COPY package.json bun.lockb* ./

# Install dependencies
RUN bun install --frozen-lockfile

# Copy source
COPY . .

# Build production bundle
RUN bun run build

# ═══════════════════════════════════════════════════════════════
#                    NGINX RUNTIME
# ═══════════════════════════════════════════════════════════════
FROM nginx:alpine AS runtime

# Copy built assets
COPY --from=builder /app/build /usr/share/nginx/html

# Copy nginx config
COPY nginx.conf /etc/nginx/conf.d/default.conf

# Create non-root user
RUN chown -R nginx:nginx /usr/share/nginx/html

EXPOSE 80

CMD ["nginx", "-g", "daemon off;"]
