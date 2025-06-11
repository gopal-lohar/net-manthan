# Dockerfile
FROM nginx:alpine

# Copy nginx configuration
COPY nginx.conf /etc/nginx/nginx.conf

# Copy the file generation script
COPY generate_files.sh /docker-entrypoint.d/01-generate-files.sh

# Make the script executable
RUN chmod +x /docker-entrypoint.d/01-generate-files.sh

# Create directories for different test scenarios
RUN mkdir -p /usr/share/nginx/html/files \
    /usr/share/nginx/html/no-resume \
    /usr/share/nginx/html/auth \
    /usr/share/nginx/html/slow \
    /usr/share/nginx/html/cookie

EXPOSE 8080