# VORTEX UI - Production Image
FROM node:20-alpine AS builder

WORKDIR /app

COPY ui/package*.json ./
RUN npm ci

COPY ui/ .

# Build (adjust command based on actual UI setup)
RUN npm run build

# Runtime
FROM nginx:alpine

COPY --from=builder /app/dist /usr/share/nginx/html
COPY docker/ui/nginx.conf /etc/nginx/conf.d/default.conf

EXPOSE 11100

CMD ["nginx", "-g", "daemon off;"]