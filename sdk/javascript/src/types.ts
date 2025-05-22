/**
 * Common types and interfaces for the Microsandbox TypeScript SDK
 */

/**
 * Options for creating a sandbox
 */
export interface SandboxOptions {
  /**
   * URL of the Microsandbox server
   */
  serverUrl?: string;

  /**
   * Namespace for the sandbox
   */
  namespace?: string;

  /**
   * Name for the sandbox
   */
  name?: string;

  /**
   * API key for Microsandbox server authentication
   */
  apiKey?: string;

  /**
   * Docker image to use for the sandbox
   */
  image?: string;

  /**
   * Memory limit in MB
   */
  memory?: number;

  /**
   * CPU limit (will be rounded to nearest integer)
   */
  cpus?: number;

  /**
   * Maximum time in seconds to wait for the sandbox to start
   */
  timeout?: number;
}

/**
 * Builder pattern for SandboxOptions
 */
export class SandboxOptionsBuilder {
  private options: SandboxOptions = {};

  /**
   * Set server URL
   */
  serverUrl(serverUrl: string): SandboxOptionsBuilder {
    this.options.serverUrl = serverUrl;
    return this;
  }

  /**
   * Set namespace
   */
  namespace(namespace: string): SandboxOptionsBuilder {
    this.options.namespace = namespace;
    return this;
  }

  /**
   * Set sandbox name
   */
  name(name: string): SandboxOptionsBuilder {
    this.options.name = name;
    return this;
  }

  /**
   * Set API key
   */
  apiKey(apiKey: string): SandboxOptionsBuilder {
    this.options.apiKey = apiKey;
    return this;
  }

  /**
   * Set Docker image
   */
  image(image: string): SandboxOptionsBuilder {
    this.options.image = image;
    return this;
  }

  /**
   * Set memory limit
   */
  memory(memory: number): SandboxOptionsBuilder {
    this.options.memory = memory;
    return this;
  }

  /**
   * Set CPU limit
   */
  cpus(cpus: number): SandboxOptionsBuilder {
    this.options.cpus = cpus;
    return this;
  }

  /**
   * Set timeout
   */
  timeout(timeout: number): SandboxOptionsBuilder {
    this.options.timeout = timeout;
    return this;
  }

  /**
   * Build SandboxOptions object
   */
  build(): SandboxOptions {
    return { ...this.options };
  }
}

/**
 * Namespace for SandboxOptions
 */
export namespace SandboxOptions {
  /**
   * Create a builder for SandboxOptions
   */
  export const builder = (): SandboxOptionsBuilder =>
    new SandboxOptionsBuilder();
}

/**
 * Output line from sandbox execution
 */
export interface OutputLine {
  stream: "stdout" | "stderr";
  text: string;
}

/**
 * Output data from sandbox execution
 */
export interface OutputData {
  output?: OutputLine[];
  status?: string;
  language?: string;
  success?: boolean;
  exit_code?: number;
  command?: string;
  args?: string[];
}
