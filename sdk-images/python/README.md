# Python SDK Image

This directory contains the Dockerfile for the Python SDK image used with microsandbox.

## Features

- Latest Python version
- Common Python development packages pre-installed
- microsandbox-portal service with Python REPL support
- Non-root user for improved security

## Building the Image

To build the image, run the following command from the project root:

```bash
docker build -t msb-python -f sdk-images/python/Dockerfile .
```

The Dockerfile uses a multi-stage build that automatically compiles the portal binary with Python features enabled, so no separate build step is required.

Alternatively, you can use the provided build script:

```bash
./scripts/build_sdk_images.sh -s python
```

## Running the Container

To run the container with the portal service accessible on port 4444:

```bash
docker run -it -p 4444:4444 -e RUST_LOG=info --name msb-python msb-python
```

### Options

- `-p 4444:4444`: Maps container port 4444 to host port 4444
- `-e RUST_LOG=info`: Sets logging level for better debugging
- `--name msb-python`: Names the container for easier reference

## Accessing the Container

To access a shell inside the running container:

```bash
docker exec -it msb-python bash
```

## Stopping and Cleaning Up

```bash
# Stop the container
docker stop msb-python

# Remove the container
docker rm msb-python

# Remove the image (optional)
docker rmi msb-python
```

## Customization

### Adding Additional Python Packages

You can customize the Dockerfile to include additional Python packages:

```dockerfile
# Add this to the Dockerfile
RUN pip install --no-cache-dir \
    numpy \
    pandas \
    matplotlib
```

### Mounting Local Files

To access your local files inside the container:

```bash
docker run -it -p 4444:4444 -v $(pwd)/your_code:/home/python-user/work --name msb-python msb-python
```

## Troubleshooting

If you encounter connection issues to the portal:

1. Check the logs: `docker logs msb-python`
2. Verify the portal is running: `docker exec -it msb-python ps aux | grep portal`
3. Ensure port 4444 is available on your host machine
