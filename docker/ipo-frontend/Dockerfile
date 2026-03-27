FROM node:24-bookworm-slim AS builder
WORKDIR /app
RUN corepack enable && corepack prepare pnpm@10.33.0 --activate
COPY pnpm-workspace.yaml pnpm-lock.yaml ./
COPY ipo-frontend/package.json ipo-frontend/package.json
COPY ipo-frontend-shared/package.json ipo-frontend-shared/package.json
RUN --mount=type=cache,target=/root/.local/share/pnpm/store \
    pnpm install --frozen-lockfile --filter ipo-frontend...
COPY ipo-frontend-shared ipo-frontend-shared
COPY ipo-frontend ipo-frontend
RUN cd ipo-frontend && pnpm build

FROM node:24-bookworm-slim AS runner
WORKDIR /app
ENV NODE_ENV=production
COPY --from=builder /app/ipo-frontend/.next/standalone ./
COPY --from=builder /app/ipo-frontend/.next/static ./.next/static
COPY --from=builder /app/ipo-frontend/public ./public
ENV PORT=3000
EXPOSE 3000
CMD ["node", "ipo-frontend/server.js"]
