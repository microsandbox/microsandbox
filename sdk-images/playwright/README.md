# Playwright Docker Image

This directory contains a Dockerfile for setting up a Playwright browser testing environment.

## Features

- Playwright v1.42.1
- Chromium, Firefox, and WebKit browsers pre-installed
- TypeScript and ts-node support
- Sample test setup
- Non-root user for better security

## Building the Image

To build the Docker image, run the following command from this directory:

```bash
docker build -t playwright-test .
```

## Running the Container

To start a container with the Playwright environment, run:

```bash
docker run -it --rm -v $(pwd):/home/playwright-user/work playwright-test bash
```

This will:

- Start a container with the Playwright testing image
- Mount your current directory to the work directory in the container
- Start an interactive bash session
- Remove the container when you exit the bash session

## Running Playwright Tests

Once inside the container, you can:

1. Navigate to your project directory:

   ```bash
   cd /home/playwright-user/work
   ```

2. Install dependencies:

   ```bash
   npm install
   ```

3. Run tests:
   ```bash
   npx playwright test
   ```

## Example Project

The container includes a sample Playwright test setup in the `/home/playwright-user/work/example` directory:

```bash
cd /home/playwright-user/work/example
npm install @playwright/test
npx playwright test
```

The example test demonstrates a basic page navigation and title check.

## Customization

You can customize the Dockerfile to:

- Change the Playwright version
- Add additional npm packages
- Modify the browser configurations
- Change the user or security settings
