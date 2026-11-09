FROM node:22-alpine
ENV NODE_ENV=production PORT=8080 HOST=0.0.0.0
WORKDIR /app
COPY --chown=node:node package.json package-lock.json ./
USER node
RUN npm ci --omit=dev --ignore-scripts
COPY --chown=node:node src ./src
EXPOSE 8080
CMD ["node", "src/server.js"]
