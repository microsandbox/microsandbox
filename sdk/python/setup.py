from setuptools import find_packages, setup

setup(
    name="microsandbox",
    version="0.1.0",
    packages=find_packages(),
    description="Microsandbox Python SDK for running code in isolated sandbox environments",
    long_description=open("README.md").read(),
    long_description_content_type="text/markdown",
    author="Microsandbox Team",
    author_email="team@microsandbox.dev",
    url="https://microsandbox.dev",
    classifiers=[
        "Programming Language :: Python :: 3",
        "Programming Language :: Python :: 3.7",
        "Programming Language :: Python :: 3.8",
        "Programming Language :: Python :: 3.9",
        "Programming Language :: Python :: 3.10",
        "Programming Language :: Python :: 3.11",
        "License :: OSI Approved :: Apache Software License",
        "Operating System :: OS Independent",
        "Development Status :: 4 - Beta",
        "Intended Audience :: Developers",
        "Topic :: Software Development :: Libraries :: Python Modules",
    ],
    python_requires=">=3.7",
    install_requires=[
        "aiohttp>=3.8.0",
        # asyncio is part of the standard library since Python 3.7
    ],
    extras_require={
        "dev": [
            "pytest>=6.0.0",
            "pytest-asyncio>=0.18.0",
            "black>=22.0.0",
            "isort>=5.0.0",
            "mypy>=0.900",
            "build>=0.8.0",
            "twine>=4.0.0",
        ],
    },
)
