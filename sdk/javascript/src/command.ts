/**
 * Command execution interface for the Microsandbox TypeScript SDK.
 */

import { v4 as uuidv4 } from "uuid";
import fetch from "node-fetch";

import { CommandExecution } from "./command-execution";
import { BaseSandbox } from "./base-sandbox";

export class Command {
  private sandbox: BaseSandbox;

  /**
   * Initialize the command instance.
   *
   * @param sandboxInstance - The sandbox instance this command belongs to
   */
  constructor(sandboxInstance: BaseSandbox) {
    this.sandbox = sandboxInstance;
  }

  /**
   * Execute a shell command in the sandbox.
   *
   * @param command - The command to execute
   * @param args - Optional list of command arguments
   * @param timeout - Optional timeout in seconds
   * @returns A CommandExecution object containing the results
   * @throws Error if the sandbox is not started or execution fails
   */
  async run(
    command: string,
    args?: string[],
    timeout?: number
  ): Promise<CommandExecution> {
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
    const requestData: {
      jsonrpc: string;
      method: string;
      params: {
        sandbox: string;
        namespace: string;
        command: string;
        args: string[];
        timeout?: number;
      };
      id: string;
    } = {
      jsonrpc: "2.0",
      method: "sandbox.command.run",
      params: {
        sandbox: this.sandbox.name,
        namespace: this.sandbox.namespace,
        command,
        args: args || [],
      },
      id: uuidv4(),
    };

    // Add timeout if specified
    if (timeout !== undefined) {
      requestData.params.timeout = timeout;
    }

    try {
      const response = await fetch(`${this.sandbox.serverUrl}/api/v1/rpc`, {
        method: "POST",
        headers,
        body: JSON.stringify(requestData),
      });

      if (!response.ok) {
        const errorText = await response.text();
        throw new Error(`Failed to execute command: ${errorText}`);
      }

      const responseData = await response.json();

      if ("error" in responseData) {
        throw new Error(
          `Failed to execute command: ${responseData.error.message}`
        );
      }

      const result = responseData.result || {};

      // Create and return a CommandExecution object with the output data
      return new CommandExecution(result);
    } catch (e) {
      if (e instanceof Error) {
        throw new Error(`Failed to execute command: ${e.message}`);
      }
      throw new Error("Failed to execute command: Unknown error");
    }
  }
}
