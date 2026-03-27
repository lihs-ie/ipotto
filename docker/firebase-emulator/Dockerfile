FROM eclipse-temurin:21-jre AS java

FROM node:24-slim
COPY --from=java /opt/java/openjdk /opt/java/openjdk
ENV JAVA_HOME=/opt/java/openjdk
ENV PATH="${JAVA_HOME}/bin:${PATH}"

RUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*
RUN npm install -g firebase-tools

WORKDIR /app
COPY firebase.json .

EXPOSE 4000 8088 9099

CMD ["firebase", "emulators:start", "--project=ipotto-local"]
