# Node.js SDK Image

This directory contains the Dockerfile for the Node.js SDK image used with microsandbox.

## Features

- Node.js 20.x LTS (Latest LTS version)
- NPM package manager
- TypeScript and ts-node
- Development tools (nodemon, eslint, prettier)
- microsandbox-portal service with JavaScript REPL support
- Built-in non-root 'node' user for improved security

## Building the Image

To build the image, run the following command from the project root:

```bash
docker build -t msb-nodejs -f sdk-images/nodejs/Dockerfile .
```

The Dockerfile uses a multi-stage build that automatically compiles the portal binary with Node.js features enabled, so no separate build step is required.

Alternatively, you can use the provided build script:

```bash
./scripts/build_sdk_images.sh -s nodejs
```

## Running the Container

To run the container with the portal service accessible on port 4444:

```bash
docker run -it -p 4444:4444 -e RUST_LOG=info --name msb-nodejs msb-nodejs
```

### Options

- `-p 4444:4444`: Maps container port 4444 to host port 4444
- `-e RUST_LOG=info`: Sets logging level for better debugging
- `--name msb-nodejs`: Names the container for easier reference

## Accessing the Container

To access a shell inside the running container:

```bash
docker exec -it msb-nodejs bash
```

## Stopping and Cleaning Up

```bash
# Stop the container
docker stop msb-nodejs

# Remove the container
docker rm msb-nodejs

# Remove the image (optional)
docker rmi msb-nodejs
```

## Customization

### Adding Additional NPM Packages

You can customize the Dockerfile to include additional NPM packages:

```dockerfile
# Add this to the Dockerfile
RUN npm install -g \
    jest \
    webpack \
    webpack-cli
```

### Mounting Local Files

To access your local files inside the container:

```bash
docker run -it -p 4444:4444 -v $(pwd)/your_code:/home/node/work --name msb-nodejs msb-nodejs
```

## Troubleshooting

If you encounter connection issues to the portal:

1. Check the logs: `docker logs msb-nodejs`
2. Verify the portal is running: `docker exec -it msb-nodejs ps aux | grep portal`
3. Ensure port 4444 is available on your host machine
