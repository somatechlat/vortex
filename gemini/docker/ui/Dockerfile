# VORTEX UI Dockerfile
# Svelte 5 + Bun + Nginx
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
COPY docker/ui/nginx.conf /etc/nginx/conf.d/default.conf

# Create non-root user
RUN chown -R nginx:nginx /usr/share/nginx/html

EXPOSE 80

CMD ["nginx", "-g", "daemon off;"]
