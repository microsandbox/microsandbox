"""
Classes for retrieving and representing sandbox metrics.
"""

import uuid
from typing import Any, Dict

import aiohttp


class SandboxMetrics:
    """
    Represents metrics data about a sandbox.
    """

    def __init__(self, metrics_data: Dict[str, Any]):
        """
        Initialize a metrics instance.

        Args:
            metrics_data: Metrics data from the sandbox.metrics.get response
        """
        self._running = metrics_data.get("running", False)
        self._cpu_usage = metrics_data.get("cpu_usage", 0.0)
        self._memory_usage = metrics_data.get("memory_usage", 0)
        self._disk_usage = metrics_data.get("disk_usage", 0)
        self._raw_data = metrics_data

    @property
    def running(self) -> bool:
        """
        Check if the sandbox is running.

        Returns:
            Boolean indicating if the sandbox is running
        """
        return self._running

    @property
    def cpu_usage(self) -> float:
        """
        Get the CPU usage percentage.

        Returns:
            Float representing CPU usage percentage (0-100)
        """
        return self._cpu_usage

    @property
    def memory_usage(self) -> int:
        """
        Get the memory usage in bytes.

        Returns:
            Integer representing memory usage in bytes
        """
        return self._memory_usage

    @property
    def memory_usage_mb(self) -> float:
        """
        Get the memory usage in megabytes.

        Returns:
            Float representing memory usage in MB
        """
        return self._memory_usage / (1024 * 1024) if self._memory_usage else 0.0

    @property
    def disk_usage(self) -> int:
        """
        Get the disk usage in bytes.

        Returns:
            Integer representing disk usage in bytes
        """
        return self._disk_usage

    @property
    def disk_usage_mb(self) -> float:
        """
        Get the disk usage in megabytes.

        Returns:
            Float representing disk usage in MB
        """
        return self._disk_usage / (1024 * 1024) if self._disk_usage else 0.0

    @property
    def raw_data(self) -> Dict[str, Any]:
        """
        Get the raw metrics data.

        Returns:
            Dictionary containing all raw metrics data
        """
        return self._raw_data


class Metrics:
    """
    Metrics class for retrieving sandbox metrics information.
    """

    def __init__(self, sandbox_instance):
        """
        Initialize the metrics instance.

        Args:
            sandbox_instance: The sandbox instance these metrics belong to
        """
        self._sandbox = sandbox_instance

    async def get(self) -> SandboxMetrics:
        """
        Get the current metrics for the sandbox.

        Returns:
            A SandboxMetrics object containing metrics information

        Raises:
            RuntimeError: If the sandbox is not started or metrics retrieval fails
        """
        if not self._sandbox._is_started:
            raise RuntimeError("Sandbox is not started. Call start() first.")

        headers = {"Content-Type": "application/json"}
        if self._sandbox._api_key:
            headers["Authorization"] = f"Bearer {self._sandbox._api_key}"

        # Prepare the request data
        request_data = {
            "jsonrpc": "2.0",
            "method": "sandbox.getStatus",
            "params": {
                "namespace": self._sandbox._namespace,
                "sandbox": self._sandbox._name,
            },
            "id": str(uuid.uuid4()),
        }

        try:
            async with self._sandbox._session.post(
                f"{self._sandbox._server_url}/api/v1/rpc",
                json=request_data,
                headers=headers,
            ) as response:
                if response.status != 200:
                    error_text = await response.text()
                    raise RuntimeError(f"Failed to get sandbox metrics: {error_text}")

                response_data = await response.json()
                if "error" in response_data:
                    raise RuntimeError(
                        f"Failed to get sandbox metrics: {response_data['error']['message']}"
                    )

                result = response_data.get("result", {})

                # Extract sandbox data
                # If there are multiple sandboxes returned, find our specific one
                sandboxes = result.get("sandboxes", [])
                our_sandbox = None
                for sandbox in sandboxes:
                    if (
                        sandbox.get("name") == self._sandbox._name
                        and sandbox.get("namespace") == self._sandbox._namespace
                    ):
                        our_sandbox = sandbox
                        break

                if not our_sandbox:
                    # If no specific sandbox found, return empty metrics
                    our_sandbox = {}

                # Create and return a SandboxMetrics object
                return SandboxMetrics(our_sandbox)
        except aiohttp.ClientError as e:
            raise RuntimeError(f"Failed to get sandbox metrics: {e}")
