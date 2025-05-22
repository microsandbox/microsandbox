/**
 * Python-specific sandbox implementation for the Microsandbox TypeScript SDK.
 */

import { v4 as uuidv4 } from "uuid";
import fetch from "node-fetch";

import { BaseSandbox } from "./base-sandbox";
import { Execution } from "./execution";
import { SandboxOptions } from "./types";

export class PythonSandbox extends BaseSandbox {
  /**
   * Get the default Docker image for Python sandbox.
   *
   * @returns A string containing the Docker image name and tag
   */
  async getDefaultImage(): Promise<string> {
    return "microsandbox/python";
  }

  /**
   * Execute Python code in the sandbox.
   *
   * @param code - Python code to execute
   * @param options - Optional execution options like timeout
   * @returns An Execution object that represents the executed code
   * @throws Error if the sandbox is not started or execution fails
   */
  async run(code: string, options?: { timeout?: number }): Promise<Execution> {
    if (!this._isStarted) {
      throw new Error("Sandbox is not started. Call start() first.");
    }

    const headers: Record<string, string> = {
      "Content-Type": "application/json",
    };

    if (this._apiKey) {
      headers["Authorization"] = `Bearer ${this._apiKey}`;
    }

    const requestData: {
      jsonrpc: string;
      method: string;
      params: {
        sandbox: string;
        namespace: string;
        language: string;
        code: string;
        timeout?: number;
      };
      id: string;
    } = {
      jsonrpc: "2.0",
      method: "sandbox.repl.run",
      params: {
        sandbox: this._name,
        namespace: this._namespace,
        language: "python",
        code,
      },
      id: uuidv4(),
    };

    // Add timeout if specified in options
    if (options?.timeout !== undefined) {
      requestData.params.timeout = options.timeout;
    }

    try {
      const response = await fetch(`${this._serverUrl}/api/v1/rpc`, {
        method: "POST",
        headers,
        body: JSON.stringify(requestData),
      });

      if (!response.ok) {
        const errorText = await response.text();
        throw new Error(`Failed to execute code: ${errorText}`);
      }

      const responseData = await response.json();

      if ("error" in responseData) {
        throw new Error(
          `Failed to execute code: ${responseData.error.message}`
        );
      }

      const result = responseData.result || {};

      // Create and return an Execution object with the output data
      return new Execution(result);
    } catch (e) {
      if (e instanceof Error) {
        throw new Error(`Failed to execute code: ${e.message}`);
      }
      throw new Error("Failed to execute code: Unknown error");
    }
  }

  /**
   * Create and initialize a new PythonSandbox instance.
   *
   * @param options - Configuration options for the sandbox
   * @returns A Promise resolving to a new PythonSandbox instance
   */
  static async create(options?: SandboxOptions): Promise<PythonSandbox> {
    return BaseSandbox.createBase(PythonSandbox, options);
  }
}
