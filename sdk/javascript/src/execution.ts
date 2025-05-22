/**
 * Class representing code execution results in a sandbox environment.
 */

import { OutputData, OutputLine } from "./types";

export class Execution {
  private outputLines: OutputLine[] = [];
  private _status: string = "unknown";
  private _language: string = "unknown";
  private _hasError: boolean = false;

  /**
   * Initialize an execution instance.
   *
   * @param outputData - Output data from the sandbox.repl.run response
   */
  constructor(outputData?: OutputData) {
    if (outputData) {
      this.processOutputData(outputData);
    }
  }

  /**
   * Process output data from the sandbox.repl.run response.
   *
   * @param outputData - Dictionary containing the output data
   */
  private processOutputData(outputData: OutputData): void {
    // Extract output lines from the response
    this.outputLines = outputData.output || [];

    // Store additional metadata that might be useful
    this._status = outputData.status || "unknown";
    this._language = outputData.language || "unknown";

    // Check for errors in the output or status
    if (this._status === "error" || this._status === "exception") {
      this._hasError = true;
    } else {
      // Check if there's any stderr output
      for (const line of this.outputLines) {
        if (line.stream === "stderr" && line.text) {
          this._hasError = true;
          break;
        }
      }
    }
  }

  /**
   * Get the standard output from the execution.
   *
   * @returns String containing the stdout output of the execution
   */
  async output(): Promise<string> {
    // Combine the stdout output lines into a single string
    let outputText = "";
    for (const line of this.outputLines) {
      if (line.stream === "stdout") {
        outputText += line.text + "\n";
      }
    }

    return outputText.trim();
  }

  /**
   * Get the error output from the execution.
   *
   * @returns String containing the stderr output of the execution
   */
  async error(): Promise<string> {
    // Combine the stderr output lines into a single string
    let errorText = "";
    for (const line of this.outputLines) {
      if (line.stream === "stderr") {
        errorText += line.text + "\n";
      }
    }

    return errorText.trim();
  }

  /**
   * Check if the execution contains an error.
   *
   * @returns Boolean indicating whether the execution encountered an error
   */
  hasError(): boolean {
    return this._hasError;
  }

  /**
   * Get the status of the execution.
   *
   * @returns String containing the execution status (e.g., "success")
   */
  get status(): string {
    return this._status;
  }

  /**
   * Get the language used for the execution.
   *
   * @returns String containing the execution language (e.g., "python")
   */
  get language(): string {
    return this._language;
  }
}
