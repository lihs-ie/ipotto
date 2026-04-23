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
# NEXT_PUBLIC_* env vars must be present at build time because Next.js
# inlines them into the client bundle. Runtime environment variables
# are ignored for client code, so CI / production must pass these as
# build args via docker-compose `build.args`.
ARG NEXT_PUBLIC_API_URL
ARG NEXT_PUBLIC_API_BASE_URL
ARG NEXT_PUBLIC_FIREBASE_API_KEY
ARG NEXT_PUBLIC_FIREBASE_AUTH_DOMAIN
ARG NEXT_PUBLIC_FIREBASE_PROJECT_ID
ARG NEXT_PUBLIC_FIREBASE_AUTH_EMULATOR_HOST
ARG NEXT_PUBLIC_FIRESTORE_EMULATOR_HOST
ENV NEXT_PUBLIC_API_URL=${NEXT_PUBLIC_API_URL}
ENV NEXT_PUBLIC_API_BASE_URL=${NEXT_PUBLIC_API_BASE_URL}
ENV NEXT_PUBLIC_FIREBASE_API_KEY=${NEXT_PUBLIC_FIREBASE_API_KEY}
ENV NEXT_PUBLIC_FIREBASE_AUTH_DOMAIN=${NEXT_PUBLIC_FIREBASE_AUTH_DOMAIN}
ENV NEXT_PUBLIC_FIREBASE_PROJECT_ID=${NEXT_PUBLIC_FIREBASE_PROJECT_ID}
ENV NEXT_PUBLIC_FIREBASE_AUTH_EMULATOR_HOST=${NEXT_PUBLIC_FIREBASE_AUTH_EMULATOR_HOST}
ENV NEXT_PUBLIC_FIRESTORE_EMULATOR_HOST=${NEXT_PUBLIC_FIRESTORE_EMULATOR_HOST}
RUN cd ipo-frontend && pnpm build

FROM node:24-bookworm-slim AS runner
WORKDIR /app
ENV NODE_ENV=production
COPY --from=builder /app/ipo-frontend/.next/standalone ./
COPY --from=builder /app/ipo-frontend/.next/static ./ipo-frontend/.next/static
COPY --from=builder /app/ipo-frontend/public ./ipo-frontend/public
ENV PORT=3000
EXPOSE 3000
CMD ["node", "ipo-frontend/server.js"]
