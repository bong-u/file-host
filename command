docker run --name file-host \
  -p 8081:8081 \
  -v "$(pwd)/app:/app" \
  -w /app -it \
  rust:alpine

apk add --no-cache build-base musl-dev libc-dev zlib-dev
