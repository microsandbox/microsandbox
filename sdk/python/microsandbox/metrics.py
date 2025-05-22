"""
Metrics interface for the Microsandbox Python SDK.
"""

import uuid
from typing import Optional


class Metrics:
    """
    Metrics class for retrieving resource metrics for a sandbox.

    This class provides methods to access specific metrics (CPU, memory, disk)
    for the sandbox instance it's attached to.
    """

    def __init__(self, sandbox_instance):
        """
        Initialize the metrics instance.

        Args:
            sandbox_instance: The sandbox instance this metrics object belongs to
        """
        self._sandbox = sandbox_instance

    async def _get_metrics(self) -> dict:
        """
        Internal method to fetch current metrics from the server.

        Returns:
            A dictionary containing the metrics data for the sandbox

        Raises:
            RuntimeError: If the request to the server fails
        """
        if not self._sandbox._is_started:
            raise RuntimeError("Sandbox is not started. Call start() first.")

        headers = {"Content-Type": "application/json"}
        if self._sandbox._api_key:
            headers["Authorization"] = f"Bearer {self._sandbox._api_key}"

        # Prepare the request data
        request_data = {
            "jsonrpc": "2.0",
            "method": "sandbox.metrics.get",
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
                sandboxes = result.get("sandboxes", [])

                # We expect exactly one sandbox in the response (our own)
                if not sandboxes:
                    return {}

                # Return the first (and should be only) sandbox data
                return sandboxes[0]
        except Exception as e:
            raise RuntimeError(f"Failed to get sandbox metrics: {e}")

    async def all(self) -> dict:
        """
        Get all metrics for the current sandbox.

        Returns:
            A dictionary containing all metrics for the sandbox:
            {
                "name": str,
                "namespace": str,
                "running": bool,
                "cpu_usage": Optional[float],
                "memory_usage": Optional[int],
                "disk_usage": Optional[int]
            }

        Raises:
            RuntimeError: If the sandbox is not started or if the request fails
        """
        return await self._get_metrics()

    async def cpu(self) -> Optional[float]:
        """
        Get CPU usage percentage for the current sandbox.

        Returns:
            CPU usage as a percentage (0-100) or None if not available.
            May return 0.0 for idle sandboxes or when metrics are not precise.

        Raises:
            RuntimeError: If the sandbox is not started or if the request fails
        """
        metrics = await self._get_metrics()
        return metrics.get("cpu_usage")

    async def memory(self) -> Optional[int]:
        """
        Get memory usage for the current sandbox.

        Returns:
            Memory usage in MiB or None if not available

        Raises:
            RuntimeError: If the sandbox is not started or if the request fails
        """
        metrics = await self._get_metrics()
        return metrics.get("memory_usage")

    async def disk(self) -> Optional[int]:
        """
        Get disk usage for the current sandbox.

        Returns:
            Disk usage in bytes or None if not available

        Raises:
            RuntimeError: If the sandbox is not started or if the request fails
        """
        metrics = await self._get_metrics()
        return metrics.get("disk_usage")

    async def is_running(self) -> bool:
        """
        Check if the sandbox is currently running.

        Returns:
            True if the sandbox is running, False otherwise

        Raises:
            RuntimeError: If the request to the server fails
        """
        metrics = await self._get_metrics()
        return metrics.get("running", False)
