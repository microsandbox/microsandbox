/**
 * Base sandbox implementation for the Microsandbox TypeScript SDK.
 */

import { v4 as uuidv4 } from "uuid";
import fetch from "node-fetch";
import * as dotenv from "dotenv";

import { Command } from "./command";
import { Metrics } from "./metrics";
import { Execution } from "./execution";
import { SandboxOptions } from "./types";

export abstract class BaseSandbox {
  protected _serverUrl: string;
  protected _namespace: string;
  protected _name: string;
  protected _apiKey: string | undefined;
  protected _isStarted: boolean = false;

  /**
   * Initialize a base sandbox instance.
   *
   * @param options - Configuration options for the sandbox
   */
  constructor(options?: SandboxOptions) {
    // Try to load from .env if MSB_API_KEY is not already set
    if (!process.env.MSB_API_KEY) {
      try {
        dotenv.config();
      } catch (error) {
        // Ignore errors if .env file doesn't exist
      }
    }

    this._serverUrl =
      options?.serverUrl ||
      process.env.MSB_SERVER_URL ||
      "http://127.0.0.1:5555";
    this._namespace = options?.namespace || "default";
    this._name = options?.name || `sandbox-${uuidv4().substring(0, 8)}`;
    this._apiKey = options?.apiKey || process.env.MSB_API_KEY;
  }

  /**
   * Create and initialize a new sandbox instance.
   *
   * This is the base implementation that subclasses should call.
   *
   * @param ctor - The constructor function for the subclass
   * @param options - Configuration options for the sandbox
   * @returns A Promise resolving to a new sandbox instance
   */
  protected static async createBase<T extends BaseSandbox>(
    ctor: new (options?: SandboxOptions) => T,
    options?: SandboxOptions
  ): Promise<T> {
    // Try to load from .env if MSB_API_KEY is not already set
    if (!process.env.MSB_API_KEY) {
      try {
        dotenv.config();
      } catch (error) {
        // Ignore errors if .env file doesn't exist
      }
    }

    const sandbox = new ctor(options);

    // Start the sandbox
    await sandbox.start(
      options?.image,
      options?.memory,
      options?.cpus,
      options?.timeout
    );

    return sandbox;
  }

  // Abstract static method signature that subclasses must implement
  static create(options?: SandboxOptions): Promise<BaseSandbox> {
    throw new Error("Static method 'create' must be implemented by subclass");
  }

  /**
   * Get the default Docker image for this sandbox type.
   *
   * @returns A string containing the Docker image name and tag
   */
  abstract getDefaultImage(): Promise<string>;

  /**
   * Start the sandbox container.
   *
   * @param image - Docker image to use for the sandbox (defaults to language-specific image)
   * @param memory - Memory limit in MB
   * @param cpus - CPU limit (will be rounded to nearest integer)
   * @param timeout - Maximum time in seconds to wait for the sandbox to start (default: 180 seconds)
   *
   * @throws Error if the sandbox fails to start
   * @throws Error if the sandbox doesn't start within the specified timeout
   */
  async start(
    image?: string,
    memory: number = 512,
    cpus: number = 1.0,
    timeout: number = 180.0
  ): Promise<void> {
    if (this._isStarted) {
      return;
    }

    const sandboxImage = image || (await this.getDefaultImage());
    const requestData = {
      jsonrpc: "2.0",
      method: "sandbox.start",
      params: {
        namespace: this._namespace,
        sandbox: this._name,
        config: {
          image: sandboxImage,
          memory,
          cpus: Math.round(cpus),
        },
      },
      id: uuidv4(),
    };

    const headers: Record<string, string> = {
      "Content-Type": "application/json",
    };

    if (this._apiKey) {
      headers["Authorization"] = `Bearer ${this._apiKey}`;
    }

    try {
      // We don't have a client-side timeout like the Python SDK, but we can implement one if needed
      const response = await fetch(`${this._serverUrl}/api/v1/rpc`, {
        method: "POST",
        headers,
        body: JSON.stringify(requestData),
        // We'd need to use a more sophisticated fetch library for proper timeout handling
        // Node-fetch doesn't have a timeout option that works the same way as Python's aiohttp
      });

      if (!response.ok) {
        const errorText = await response.text();
        throw new Error(`Failed to start sandbox: ${errorText}`);
      }

      const responseData = await response.json();

      if ("error" in responseData) {
        throw new Error(
          `Failed to start sandbox: ${responseData.error.message}`
        );
      }

      // Check the result message - it might indicate the sandbox is still initializing
      const result = responseData.result;
      if (typeof result === "string" && result.includes("timed out waiting")) {
        // Server timed out but still started the sandbox
        // We'll log a warning but still consider it started
        console.warn(`Sandbox start warning: ${result}`);
      }

      this._isStarted = true;
    } catch (e) {
      if (e instanceof Error) {
        if (e.message.includes("timeout")) {
          throw new Error(
            `Timed out waiting for sandbox to start after ${timeout} seconds`
          );
        }
        throw new Error(
          `Failed to communicate with Microsandbox server: ${e.message}`
        );
      }
      throw new Error(
        "Failed to communicate with Microsandbox server: Unknown error"
      );
    }
  }

  /**
   * Stop the sandbox container.
   *
   * @throws Error if the sandbox fails to stop
   */
  async stop(): Promise<void> {
    if (!this._isStarted) {
      return;
    }

    const requestData = {
      jsonrpc: "2.0",
      method: "sandbox.stop",
      params: {
        namespace: this._namespace,
        sandbox: this._name,
      },
      id: uuidv4(),
    };

    const headers: Record<string, string> = {
      "Content-Type": "application/json",
    };

    if (this._apiKey) {
      headers["Authorization"] = `Bearer ${this._apiKey}`;
    }

    try {
      const response = await fetch(`${this._serverUrl}/api/v1/rpc`, {
        method: "POST",
        headers,
        body: JSON.stringify(requestData),
      });

      if (!response.ok) {
        const errorText = await response.text();
        throw new Error(`Failed to stop sandbox: ${errorText}`);
      }

      const responseData = await response.json();

      if ("error" in responseData) {
        throw new Error(
          `Failed to stop sandbox: ${responseData.error.message}`
        );
      }

      this._isStarted = false;
    } catch (e) {
      if (e instanceof Error) {
        throw new Error(
          `Failed to communicate with Microsandbox server: ${e.message}`
        );
      }
      throw new Error(
        "Failed to communicate with Microsandbox server: Unknown error"
      );
    }
  }

  /**
   * Execute code in the sandbox.
   *
   * @param code - Code to execute
   * @returns An Execution object representing the executed code
   * @throws Error if execution fails
   */
  abstract run(code: string): Promise<Execution>;

  /**
   * Access the command namespace for executing shell commands in the sandbox.
   */
  get command(): Command {
    return new Command(this);
  }

  /**
   * Access the metrics namespace for retrieving sandbox metrics.
   */
  get metrics(): Metrics {
    return new Metrics(this);
  }

  /**
   * Check if the sandbox is started.
   */
  get isStarted(): boolean {
    return this._isStarted;
  }

  /**
   * Get the server URL.
   */
  get serverUrl(): string {
    return this._serverUrl;
  }

  /**
   * Get the namespace.
   */
  get namespace(): string {
    return this._namespace;
  }

  /**
   * Get the sandbox name.
   */
  get name(): string {
    return this._name;
  }

  /**
   * Get the API key.
   */
  get apiKey(): string | undefined {
    return this._apiKey;
  }
}
