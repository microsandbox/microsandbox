# JavaScript Docker Image

This directory contains a Dockerfile for setting up a Node.js/JavaScript development environment.

## Features

- Node.js 20.x LTS
- NPM package manager
- TypeScript and ts-node
- Development tools (nodemon, eslint, prettier)
- Non-root user for better security

## Building the Image

To build the Docker image, run the following command from this directory:

```bash
docker build -t node-dev .
```

## Running the Container

To start a container with the Node.js environment, run:

```bash
docker run -it --rm -p 3000:3000 -v $(pwd):/home/node-user/work node-dev bash
```

This will:

- Start a container with the Node.js development image
- Map port 3000 from the container to your host (for web applications)
- Mount your current directory to the work directory in the container
- Start an interactive bash session
- Remove the container when you exit the bash session

## Developing JavaScript/TypeScript Applications

Once inside the container, you can:

- Create new Node.js projects or use existing ones
- Run scripts with `node` or `npm run`
- Use TypeScript with `tsc` or `ts-node`
- Use development tools like `eslint` and `prettier`

## Customization

You can customize the Dockerfile to:

- Change the Node.js version
- Add additional npm packages by modifying the `npm install -g` command
- Add additional system dependencies by modifying the `apt-get install` command
- Change the user or security settings
