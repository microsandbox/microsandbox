FROM mcr.microsoft.com/playwright:v1.42.1-focal

# Set environment variables
ENV NODE_ENV=development \
    NPM_CONFIG_LOGLEVEL=info

# Create a non-root user to run tests
ARG USER_NAME="playwright-user"
ARG USER_UID="1000"
ARG USER_GID="100"

# Create non-root user and necessary directories
RUN useradd -m -s /bin/bash -N -u $USER_UID $USER_NAME \
    && mkdir -p /home/$USER_NAME/work \
    && chown -R $USER_NAME:$USER_GID /home/$USER_NAME

# Install additional npm packages
RUN npm install -g npm@latest \
    typescript \
    ts-node

# Switch to the non-root user
USER $USER_NAME
WORKDIR /home/$USER_NAME/work

# Create a sample setup to demonstrate Playwright
RUN mkdir -p /home/$USER_NAME/work/example && \
    echo '{ "name": "playwright-example", "version": "1.0.0", "scripts": { "test": "playwright test" } }' > /home/$USER_NAME/work/example/package.json && \
    mkdir -p /home/$USER_NAME/work/example/tests && \
    echo 'import { test, expect } from "@playwright/test";\ntest("example test", async ({ page }) => {\n  await page.goto("https://example.com");\n  await expect(page).toHaveTitle(/Example Domain/);\n});' > /home/$USER_NAME/work/example/tests/example.spec.ts && \
    mkdir -p /home/$USER_NAME/work/example/playwright-tests && \
    echo 'import { defineConfig } from "@playwright/test";\nexport default defineConfig({\n  testDir: "./tests",\n  use: {\n    headless: true,\n  },\n});' > /home/$USER_NAME/work/example/playwright.config.ts

# Set a command that keeps the container running
CMD ["bash", "-c", "echo Playwright environment ready && tail -f /dev/null"]
