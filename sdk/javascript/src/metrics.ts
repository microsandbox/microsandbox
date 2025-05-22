/**
 * Metrics interface for the Microsandbox TypeScript SDK.
 */

import { v4 as uuidv4 } from "uuid";
import fetch from "node-fetch";

import { BaseSandbox } from "./base-sandbox";

export class Metrics {
  private sandbox: BaseSandbox;

  /**
   * Initialize the metrics instance.
   *
   * @param sandboxInstance - The sandbox instance this metrics object belongs to
   */
  constructor(sandboxInstance: BaseSandbox) {
    this.sandbox = sandboxInstance;
  }

  /**
   * Internal method to fetch current metrics from the server.
   *
   * @returns A dictionary containing the metrics data for the sandbox
   * @throws Error if the request to the server fails
   */
  private async getMetrics(): Promise<any> {
    if (!this.sandbox.isStarted) {
      throw new Error("Sandbox is not started. Call start() first.");
    }

    const headers: Record<string, string> = {
      "Content-Type": "application/json",
    };

    if (this.sandbox.apiKey) {
      headers["Authorization"] = `Bearer ${this.sandbox.apiKey}`;
    }

    // Prepare the request data
    const requestData = {
      jsonrpc: "2.0",
      method: "sandbox.metrics.get",
      params: {
        namespace: this.sandbox.namespace,
        sandbox: this.sandbox.name,
      },
      id: uuidv4(),
    };

    try {
      const response = await fetch(`${this.sandbox.serverUrl}/api/v1/rpc`, {
        method: "POST",
        headers,
        body: JSON.stringify(requestData),
      });

      if (!response.ok) {
        const errorText = await response.text();
        throw new Error(`Failed to get sandbox metrics: ${errorText}`);
      }

      const responseData = await response.json();

      if ("error" in responseData) {
        throw new Error(
          `Failed to get sandbox metrics: ${responseData.error.message}`
        );
      }

      const result = responseData.result || {};
      const sandboxes = result.sandboxes || [];

      // We expect exactly one sandbox in the response (our own)
      if (!sandboxes.length) {
        return {};
      }

      // Return the first (and should be only) sandbox data
      return sandboxes[0];
    } catch (e) {
      if (e instanceof Error) {
        throw new Error(`Failed to get sandbox metrics: ${e.message}`);
      }
      throw new Error("Failed to get sandbox metrics: Unknown error");
    }
  }

  /**
   * Get all metrics for the current sandbox.
   *
   * @returns A dictionary containing all metrics for the sandbox:
   * {
   *   "name": string,
   *   "namespace": string,
   *   "running": boolean,
   *   "cpu_usage": number | null,
   *   "memory_usage": number | null,
   *   "disk_usage": number | null
   * }
   *
   * @throws Error if the sandbox is not started or if the request fails
   */
  async all(): Promise<any> {
    return await this.getMetrics();
  }

  /**
   * Get CPU usage percentage for the current sandbox.
   *
   * @returns CPU usage as a percentage (0-100) or undefined if not available.
   * May return 0.0 for idle sandboxes or when metrics are not precise.
   *
   * @throws Error if the sandbox is not started or if the request fails
   */
  async cpu(): Promise<number | undefined> {
    const metrics = await this.getMetrics();
    return metrics.cpu_usage;
  }

  /**
   * Get memory usage for the current sandbox.
   *
   * @returns Memory usage in MiB or undefined if not available
   *
   * @throws Error if the sandbox is not started or if the request fails
   */
  async memory(): Promise<number | undefined> {
    const metrics = await this.getMetrics();
    return metrics.memory_usage;
  }

  /**
   * Get disk usage for the current sandbox.
   *
   * @returns Disk usage in bytes or undefined if not available
   *
   * @throws Error if the sandbox is not started or if the request fails
   */
  async disk(): Promise<number | undefined> {
    const metrics = await this.getMetrics();
    return metrics.disk_usage;
  }

  /**
   * Check if the sandbox is currently running.
   *
   * @returns True if the sandbox is running, False otherwise
   *
   * @throws Error if the request to the server fails
   */
  async isRunning(): Promise<boolean> {
    const metrics = await this.getMetrics();
    return metrics.running || false;
  }
}
