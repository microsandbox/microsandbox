/**
 * Class representing command execution results in a sandbox environment.
 */

import { OutputData, OutputLine } from "./types";

export class CommandExecution {
  private outputLines: OutputLine[] = [];
  private _command: string = "";
  private _args: string[] = [];
  private _exitCode: number = -1;
  private _success: boolean = false;

  /**
   * Initialize a command execution instance.
   *
   * @param outputData - Output data from the sandbox.command.run response
   */
  constructor(outputData?: OutputData) {
    if (outputData) {
      this.processOutputData(outputData);
    }
  }

  /**
   * Process output data from the sandbox.command.run response.
   *
   * @param outputData - Dictionary containing the output data
   */
  private processOutputData(outputData: OutputData): void {
    // Extract output lines from the response
    this.outputLines = outputData.output || [];

    // Store command-specific metadata
    this._command = outputData.command || "";
    this._args = outputData.args || [];
    this._exitCode =
      outputData.exit_code !== undefined ? outputData.exit_code : -1;
    this._success = outputData.success || false;
  }

  /**
   * Get the standard output from the command execution.
   *
   * @returns String containing the stdout output of the command
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
   * Get the error output from the command execution.
   *
   * @returns String containing the stderr output of the command
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
   * Get the exit code of the command execution.
   *
   * @returns Integer containing the exit code
   */
  get exitCode(): number {
    return this._exitCode;
  }

  /**
   * Check if the command executed successfully.
   *
   * @returns Boolean indicating whether the command succeeded (exit code 0)
   */
  get success(): boolean {
    return this._success;
  }

  /**
   * Get the command that was executed.
   *
   * @returns String containing the command
   */
  get command(): string {
    return this._command;
  }

  /**
   * Get the arguments used for the command.
   *
   * @returns List of strings containing the command arguments
   */
  get args(): string[] {
    return this._args;
  }
}
