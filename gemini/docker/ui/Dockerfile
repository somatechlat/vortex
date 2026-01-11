FROM oven/bun:1.1.0 AS builder
WORKDIR /app/ui

COPY ui/package.json ui/bun.lock ./
RUN bun install

COPY ui ./
RUN bun run build

FROM nginx:alpine
COPY --from=builder /app/ui/dist /usr/share/nginx/html
COPY ui/nginx.conf /etc/nginx/conf.d/default.conf

EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]
