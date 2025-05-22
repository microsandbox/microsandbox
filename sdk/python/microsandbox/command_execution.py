"""
Classes representing command execution results in a sandbox environment.
"""

from typing import Any, Dict, List, Optional


class CommandExecution:
    """
    Represents a command execution in a sandbox environment.

    This class provides access to the results and output of a command
    that was executed in a sandbox.
    """

    def __init__(
        self,
        output_data: Optional[Dict[str, Any]] = None,
    ):
        """
        Initialize a command execution instance.

        Args:
            output_data: Output data from the sandbox.command.run response
        """
        self._output_lines: List[Dict[str, str]] = []
        self._command = ""
        self._args: List[str] = []
        self._exit_code = -1
        self._success = False

        # Process output data if provided
        if output_data and isinstance(output_data, dict):
            self._process_output_data(output_data)

    def _process_output_data(self, output_data: Dict[str, Any]) -> None:
        """
        Process output data from the sandbox.command.run response.

        Args:
            output_data: Dictionary containing the output data
        """
        # Extract output lines from the response
        self._output_lines = output_data.get("output", [])

        # Store command-specific metadata
        self._command = output_data.get("command", "")
        self._args = output_data.get("args", [])
        self._exit_code = output_data.get("exit_code", -1)
        self._success = output_data.get("success", False)

    async def output(self) -> str:
        """
        Get the standard output from the command execution.

        Returns:
            String containing the stdout output of the command
        """
        # Combine the stdout output lines into a single string
        output_text = ""
        for line in self._output_lines:
            if isinstance(line, dict) and line.get("stream") == "stdout":
                output_text += line.get("text", "") + "\n"

        return output_text.rstrip()

    async def error(self) -> str:
        """
        Get the error output from the command execution.

        Returns:
            String containing the stderr output of the command
        """
        # Combine the stderr output lines into a single string
        error_text = ""
        for line in self._output_lines:
            if isinstance(line, dict) and line.get("stream") == "stderr":
                error_text += line.get("text", "") + "\n"

        return error_text.rstrip()

    @property
    def exit_code(self) -> int:
        """
        Get the exit code of the command execution.

        Returns:
            Integer containing the exit code
        """
        return self._exit_code

    @property
    def success(self) -> bool:
        """
        Check if the command executed successfully.

        Returns:
            Boolean indicating whether the command succeeded (exit code 0)
        """
        return self._success

    @property
    def command(self) -> str:
        """
        Get the command that was executed.

        Returns:
            String containing the command
        """
        return self._command

    @property
    def args(self) -> List[str]:
        """
        Get the arguments used for the command.

        Returns:
            List of strings containing the command arguments
        """
        return self._args
