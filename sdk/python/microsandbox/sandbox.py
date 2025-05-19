"""
Sandbox implementation for the Microsandbox Python SDK.
"""

import os
import uuid
from abc import ABC, abstractmethod
from contextlib import asynccontextmanager
from typing import Optional

import aiohttp
from dotenv import load_dotenv


class Execution:
    """
    Represents a code execution in a sandbox environment.

    This class provides access to the results and output of code
    that was executed in a sandbox.
    """

    def __init__(
        self,
        sandbox_name: str,
        namespace: str,
        execution_id: str,
        server_url: str,
        session: aiohttp.ClientSession,
        api_key: Optional[str] = None,
    ):
        """
        Initialize an execution instance.

        Args:
            sandbox_name: Name of the sandbox where execution occurred
            namespace: Namespace of the sandbox
            execution_id: Unique ID for this execution
            server_url: URL of the Microsandbox server
            session: HTTP session for API requests
            api_key: Optional API key for authentication
        """
        self._sandbox_name = sandbox_name
        self._namespace = namespace
        self._execution_id = execution_id
        self._server_url = server_url
        self._session = session
        self._api_key = api_key
        self._output_lines = []
        self._output_fetched = False

    async def _fetch_output(self) -> None:
        """
        Fetch the output from the execution.

        Raises:
            RuntimeError: If fetching output fails
        """
        if not self._execution_id:
            return

        headers = {"Content-Type": "application/json"}
        if self._api_key:
            headers["Authorization"] = f"Bearer {self._api_key}"

        request_data = {
            "jsonrpc": "2.0",
            "method": "sandbox.repl.getOutput",
            "params": {
                "sandbox": self._sandbox_name,
                "namespace": self._namespace,
                "execution_id": self._execution_id,
            },
            "id": str(uuid.uuid4()),
        }

        try:
            async with self._session.post(
                f"{self._server_url}/api/v1/rpc",
                json=request_data,
                headers=headers,
            ) as response:
                if response.status != 200:
                    error_text = await response.text()
                    raise RuntimeError(f"Failed to get output: {error_text}")

                response_data = await response.json()
                if "error" in response_data:
                    raise RuntimeError(
                        f"Failed to get output: {response_data['error']['message']}"
                    )

                result = response_data.get("result", {})
                self._output_lines = result.get("lines", [])
                self._output_fetched = True
        except aiohttp.ClientError as e:
            raise RuntimeError(f"Failed to fetch output: {e}")

    async def output(self) -> str:
        """
        Get the output from the execution.

        Returns:
            String containing the output of the execution

        Raises:
            RuntimeError: If fetching output fails
        """
        # Ensure we have the latest output
        if not self._output_fetched:
            await self._fetch_output()

        # Combine the output lines into a single string
        output_text = ""
        for line in self._output_lines:
            if line.get("stream") == "stdout":
                output_text += line.get("text", "") + "\n"

        return output_text.rstrip()


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
        self, image: Optional[str] = None, memory: int = 512, cpus: float = 1.0
    ) -> None:
        """
        Start the sandbox container.

        Args:
            image: Docker image to use for the sandbox (defaults to language-specific image)
            memory: Memory limit in MB
            cpus: CPU limit (will be rounded to nearest integer)

        Raises:
            RuntimeError: If the sandbox fails to start
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
            async with self._session.post(
                f"{self._server_url}/api/v1/rpc",
                json=request_data,
                headers=headers,
            ) as response:
                if response.status != 200:
                    error_text = await response.text()
                    raise RuntimeError(f"Failed to start sandbox: {error_text}")

                response_data = await response.json()
                if "error" in response_data:
                    raise RuntimeError(
                        f"Failed to start sandbox: {response_data['error']['message']}"
                    )

                self._is_started = True
        except aiohttp.ClientError as e:
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


class PythonSandbox(BaseSandbox):
    """
    Python-specific sandbox for executing Python code.
    """

    async def get_default_image(self) -> str:
        """
        Get the default Docker image for Python sandbox.

        Returns:
            A string containing the Docker image name and tag
        """
        return "appcypher/msb-python"

    async def run(self, code: str) -> Execution:
        """
        Execute Python code in the sandbox.

        Args:
            code: Python code to execute

        Returns:
            An Execution object that represents the executed code

        Raises:
            RuntimeError: If the sandbox is not started or execution fails
        """
        if not self._is_started:
            raise RuntimeError("Sandbox is not started. Call start() first.")

        headers = {"Content-Type": "application/json"}
        if self._api_key:
            headers["Authorization"] = f"Bearer {self._api_key}"

        request_data = {
            "jsonrpc": "2.0",
            "method": "sandbox.repl.run",
            "params": {
                "sandbox": self._sandbox_name,
                "namespace": self._namespace,
                "language": "python",
                "code": code,
            },
            "id": str(uuid.uuid4()),
        }

        try:
            async with self._session.post(
                f"{self._server_url}/api/v1/rpc",
                json=request_data,
                headers=headers,
            ) as response:
                if response.status != 200:
                    error_text = await response.text()
                    raise RuntimeError(f"Failed to execute code: {error_text}")

                response_data = await response.json()
                if "error" in response_data:
                    raise RuntimeError(
                        f"Failed to execute code: {response_data['error']['message']}"
                    )

                result = response_data.get("result", {})
                execution_id = result.get("execution_id")

                # Create and return an Execution object
                return Execution(
                    sandbox_name=self._sandbox_name,
                    namespace=self._namespace,
                    execution_id=execution_id,
                    server_url=self._server_url,
                    session=self._session,
                    api_key=self._api_key,
                )
        except aiohttp.ClientError as e:
            raise RuntimeError(f"Failed to execute code: {e}")
