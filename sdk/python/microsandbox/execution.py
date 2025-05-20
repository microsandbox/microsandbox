"""
Classes representing code execution results in a sandbox environment.
"""

from typing import Any, Dict, List, Optional


class Execution:
    """
    Represents a code execution in a sandbox environment.

    This class provides access to the results and output of code
    that was executed in a sandbox.
    """

    def __init__(
        self,
        output_data: Optional[Dict[str, Any]] = None,
    ):
        """
        Initialize an execution instance.

        Args:
            output_data: Output data from the sandbox.repl.run response
        """
        self._output_lines: List[Dict[str, str]] = []
        self._status = "unknown"
        self._language = "unknown"
        self._has_error = False

        # Process output data if provided
        if output_data and isinstance(output_data, dict):
            self._process_output_data(output_data)

    def _process_output_data(self, output_data: Dict[str, Any]) -> None:
        """
        Process output data from the sandbox.repl.run response.

        Args:
            output_data: Dictionary containing the output data
        """
        # Extract output lines from the response
        self._output_lines = output_data.get("output", [])

        # Store additional metadata that might be useful
        self._status = output_data.get("status", "unknown")
        self._language = output_data.get("language", "unknown")

        # Check for errors in the output or status
        if self._status == "error" or self._status == "exception":
            self._has_error = True
        else:
            # Check if there's any stderr output
            for line in self._output_lines:
                if (
                    isinstance(line, dict)
                    and line.get("stream") == "stderr"
                    and line.get("text")
                ):
                    self._has_error = True
                    break

    async def output(self) -> str:
        """
        Get the standard output from the execution.

        Returns:
            String containing the stdout output of the execution
        """
        # Combine the stdout output lines into a single string
        output_text = ""
        for line in self._output_lines:
            if isinstance(line, dict) and line.get("stream") == "stdout":
                output_text += line.get("text", "") + "\n"

        return output_text.rstrip()

    async def error(self) -> str:
        """
        Get the error output from the execution.

        Returns:
            String containing the stderr output of the execution
        """
        # Combine the stderr output lines into a single string
        error_text = ""
        for line in self._output_lines:
            if isinstance(line, dict) and line.get("stream") == "stderr":
                error_text += line.get("text", "") + "\n"

        return error_text.rstrip()

    def has_error(self) -> bool:
        """
        Check if the execution contains an error.

        Returns:
            Boolean indicating whether the execution encountered an error
        """
        return self._has_error

    @property
    def status(self) -> str:
        """
        Get the status of the execution.

        Returns:
            String containing the execution status (e.g., "success")
        """
        return self._status

    @property
    def language(self) -> str:
        """
        Get the language used for the execution.

        Returns:
            String containing the execution language (e.g., "python")
        """
        return self._language
