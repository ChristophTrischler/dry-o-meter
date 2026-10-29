# Step 1: Build the frontend
FROM node:22-alpine AS builder

WORKDIR /app

# Copy dependency configuration and install dependencies
COPY package.json ./
RUN npm install

# Copy source code and build the production assets
COPY . .
RUN npm run build

# Step 2: Serve the frontend using Nginx
FROM nginx:alpine

# Copy the custom Nginx configuration
COPY nginx.conf /etc/nginx/conf.d/default.conf

# Copy the built assets to the default Nginx html directory
COPY --from=builder /app/dist /usr/share/nginx/html

EXPOSE 80

CMD ["nginx", "-g", "daemon off;"]
