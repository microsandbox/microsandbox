"""
Command execution interface for the Microsandbox Python SDK.
"""

import uuid
from typing import List, Optional

import aiohttp

from .command_execution import CommandExecution


class Command:
    """
    Command class for executing shell commands in a sandbox.
    """

    def __init__(self, sandbox_instance):
        """
        Initialize the command instance.

        Args:
            sandbox_instance: The sandbox instance this command belongs to
        """
        self._sandbox = sandbox_instance

    async def run(
        self,
        command: str,
        args: Optional[List[str]] = None,
        timeout: Optional[int] = None,
    ) -> CommandExecution:
        """
        Execute a shell command in the sandbox.

        Args:
            command: The command to execute
            args: Optional list of command arguments
            timeout: Optional timeout in seconds

        Returns:
            A CommandExecution object containing the results

        Raises:
            RuntimeError: If the sandbox is not started or execution fails
        """
        if not self._sandbox._is_started:
            raise RuntimeError("Sandbox is not started. Call start() first.")

        if args is None:
            args = []

        headers = {"Content-Type": "application/json"}
        if self._sandbox._api_key:
            headers["Authorization"] = f"Bearer {self._sandbox._api_key}"

        # Prepare the request data
        request_data = {
            "jsonrpc": "2.0",
            "method": "sandbox.command.execute",
            "params": {
                "sandbox": self._sandbox._sandbox_name,
                "namespace": self._sandbox._namespace,
                "command": command,
                "args": args,
            },
            "id": str(uuid.uuid4()),
        }

        # Add timeout if specified
        if timeout is not None:
            request_data["params"]["timeout"] = timeout

        try:
            async with self._sandbox._session.post(
                f"{self._sandbox._server_url}/api/v1/rpc",
                json=request_data,
                headers=headers,
            ) as response:
                if response.status != 200:
                    error_text = await response.text()
                    raise RuntimeError(f"Failed to execute command: {error_text}")

                response_data = await response.json()
                if "error" in response_data:
                    raise RuntimeError(
                        f"Failed to execute command: {response_data['error']['message']}"
                    )

                result = response_data.get("result", {})

                # Create and return a CommandExecution object with the output data
                return CommandExecution(output_data=result)
        except aiohttp.ClientError as e:
            raise RuntimeError(f"Failed to execute command: {e}")
