FROM node:24-bookworm-slim AS builder
WORKDIR /app
RUN corepack enable && corepack prepare pnpm@10.33.0 --activate
COPY pnpm-workspace.yaml pnpm-lock.yaml ./
COPY ipo-browser/package.json ipo-browser/package.json
COPY ipo-frontend-shared/package.json ipo-frontend-shared/package.json
RUN --mount=type=cache,target=/root/.local/share/pnpm/store \
    pnpm install --frozen-lockfile --filter ipo-browser...
COPY ipo-frontend-shared ipo-frontend-shared
COPY ipo-browser/tsconfig.json ipo-browser/tsconfig.json
COPY ipo-browser/src ipo-browser/src
RUN cd ipo-browser && pnpm build

FROM node:24-bookworm-slim
WORKDIR /app
RUN corepack enable && corepack prepare pnpm@10.33.0 --activate
COPY --from=builder /app/pnpm-workspace.yaml /app/pnpm-lock.yaml ./
COPY --from=builder /app/ipo-browser/package.json ipo-browser/package.json
COPY --from=builder /app/ipo-frontend-shared/package.json ipo-frontend-shared/package.json
RUN --mount=type=cache,target=/root/.local/share/pnpm/store \
    pnpm install --frozen-lockfile --filter ipo-browser... --prod
# Install browsers after pnpm so the Playwright version used here matches
# the package pinned in package.json / pnpm-lock.yaml. Using `pnpm exec`
# also guarantees we do not silently pull a different Playwright release
# from npx.
RUN cd ipo-browser \
    && pnpm exec playwright install --with-deps chromium chromium-headless-shell
COPY --from=builder /app/ipo-frontend-shared ipo-frontend-shared
COPY --from=builder /app/ipo-browser/dist ipo-browser/dist
ENV PORT=8081
EXPOSE 8081
WORKDIR /app/ipo-browser
CMD ["node", "dist/index.js"]
