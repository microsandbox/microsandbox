"""
Base sandbox implementation for the Microsandbox Python SDK.
"""

import asyncio
import os
import uuid
from abc import ABC, abstractmethod
from contextlib import asynccontextmanager
from typing import Optional

import aiohttp
from dotenv import load_dotenv

from .command import Command
from .metrics import Metrics


class BaseSandbox(ABC):
    """
    Base sandbox environment for executing code safely.

    This class provides the base interface for interacting with the Microsandbox server.
    It handles common functionality like sandbox creation, management, and communication.
    """

    def __init__(
        self,
        server_url: str = None,
        namespace: str = "default",
        sandbox_name: Optional[str] = None,
        api_key: Optional[str] = None,
    ):
        """
        Initialize a base sandbox instance.

        Args:
            server_url: URL of the Microsandbox server. If not provided, will check MSB_SERVER_URL environment variable, then fall back to default.
            namespace: Namespace for the sandbox
            sandbox_name: Optional name for the sandbox. If not provided, a random name will be generated.
            api_key: API key for Microsandbox server authentication. If not provided, it will be read from MSB_API_KEY environment variable.
        """
        load_dotenv()

        self._server_url = server_url or os.environ.get(
            "MSB_SERVER_URL", "http://127.0.0.1:5555"
        )
        self._namespace = namespace
        self._sandbox_name = sandbox_name or f"sandbox-{uuid.uuid4().hex[:8]}"
        self._api_key = api_key or os.environ.get("MSB_API_KEY")
        self._session = None
        self._is_started = False

    @abstractmethod
    async def get_default_image(self) -> str:
        """
        Get the default Docker image for this sandbox type.

        Returns:
            A string containing the Docker image name and tag
        """
        pass

    @classmethod
    @asynccontextmanager
    async def create(
        cls,
        server_url: str = None,
        namespace: str = "default",
        sandbox_name: Optional[str] = None,
        api_key: Optional[str] = None,
    ):
        """
        Create and initialize a new sandbox as an async context manager.

        Args:
            server_url: URL of the Microsandbox server. If not provided, will check MSB_SERVER_URL environment variable, then fall back to default.
            namespace: Namespace for the sandbox
            sandbox_name: Optional name for the sandbox. If not provided, a random name will be generated.
            api_key: API key for Microsandbox server authentication. If not provided, it will be read from MSB_API_KEY environment variable.

        Returns:
            An instance of the sandbox ready for use
        """
        sandbox = cls(
            server_url=server_url,
            namespace=namespace,
            sandbox_name=sandbox_name,
            api_key=api_key,
        )
        try:
            # Create HTTP session
            sandbox._session = aiohttp.ClientSession()
            # Start the sandbox
            await sandbox.start()
            yield sandbox
        finally:
            # Stop the sandbox
            await sandbox.stop()
            # Close the HTTP session
            if sandbox._session:
                await sandbox._session.close()
                sandbox._session = None

    async def start(
        self,
        image: Optional[str] = None,
        memory: int = 512,
        cpus: float = 1.0,
        timeout: float = 180.0,
    ) -> None:
        """
        Start the sandbox container.

        Args:
            image: Docker image to use for the sandbox (defaults to language-specific image)
            memory: Memory limit in MB
            cpus: CPU limit (will be rounded to nearest integer)
            timeout: Maximum time in seconds to wait for the sandbox to start (default: 180 seconds)

        Raises:
            RuntimeError: If the sandbox fails to start
            TimeoutError: If the sandbox doesn't start within the specified timeout
        """
        if self._is_started:
            return

        sandbox_image = image or await self.get_default_image()
        request_data = {
            "jsonrpc": "2.0",
            "method": "sandbox.start",
            "params": {
                "namespace": self._namespace,
                "sandbox": self._sandbox_name,
                "config": {
                    "image": sandbox_image,
                    "memory": memory,
                    "cpus": int(round(cpus)),
                },
            },
            "id": str(uuid.uuid4()),
        }

        headers = {"Content-Type": "application/json"}
        if self._api_key:
            headers["Authorization"] = f"Bearer {self._api_key}"

        try:
            # Set a client-side timeout that's a bit longer than the server-side timeout
            # to account for network latency and processing time
            client_timeout = aiohttp.ClientTimeout(total=timeout + 30)

            async with self._session.post(
                f"{self._server_url}/api/v1/rpc",
                json=request_data,
                headers=headers,
                timeout=client_timeout,
            ) as response:
                if response.status != 200:
                    error_text = await response.text()
                    raise RuntimeError(f"Failed to start sandbox: {error_text}")

                response_data = await response.json()
                if "error" in response_data:
                    raise RuntimeError(
                        f"Failed to start sandbox: {response_data['error']['message']}"
                    )

                # Check the result message - it might indicate the sandbox is still initializing
                result = response_data.get("result", "")
                if isinstance(result, str) and "timed out waiting" in result:
                    # Server timed out but still started the sandbox
                    # We'll raise a warning but still consider it started
                    import warnings

                    warnings.warn(f"Sandbox start warning: {result}")

                self._is_started = True
        except aiohttp.ClientError as e:
            if isinstance(e, asyncio.TimeoutError):
                raise TimeoutError(
                    f"Timed out waiting for sandbox to start after {timeout} seconds"
                ) from e
            raise RuntimeError(f"Failed to communicate with Microsandbox server: {e}")

    async def stop(self) -> None:
        """
        Stop the sandbox container.

        Raises:
            RuntimeError: If the sandbox fails to stop
        """
        if not self._is_started:
            return

        request_data = {
            "jsonrpc": "2.0",
            "method": "sandbox.stop",
            "params": {"namespace": self._namespace, "sandbox": self._sandbox_name},
            "id": str(uuid.uuid4()),
        }

        headers = {"Content-Type": "application/json"}
        if self._api_key:
            headers["Authorization"] = f"Bearer {self._api_key}"

        try:
            async with self._session.post(
                f"{self._server_url}/api/v1/rpc",
                json=request_data,
                headers=headers,
            ) as response:
                if response.status != 200:
                    error_text = await response.text()
                    raise RuntimeError(f"Failed to stop sandbox: {error_text}")

                response_data = await response.json()
                if "error" in response_data:
                    raise RuntimeError(
                        f"Failed to stop sandbox: {response_data['error']['message']}"
                    )

                self._is_started = False
        except aiohttp.ClientError as e:
            raise RuntimeError(f"Failed to communicate with Microsandbox server: {e}")

    @abstractmethod
    async def run(self, code: str):
        """
        Execute code in the sandbox.

        Args:
            code: Code to execute

        Returns:
            An Execution object representing the executed code

        Raises:
            RuntimeError: If execution fails
        """
        pass

    @property
    def command(self):
        """
        Access the command namespace for executing shell commands in the sandbox.

        Returns:
            A Command instance bound to this sandbox
        """
        return Command(self)

    @property
    def metrics(self):
        """
        Access the metrics namespace for retrieving sandbox metrics.

        Returns:
            A Metrics instance bound to this sandbox
        """
        return Metrics(self)
