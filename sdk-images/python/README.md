# Python Docker Image

This directory contains a Dockerfile for setting up a Python development environment.

## Features

- Python 3.11
- Common development tools (black, flake8, mypy, pytest)
- IPython for interactive development
- Non-root user for better security

## Building the Image

To build the Docker image, run the following command from this directory:

```bash
docker build -t python-dev .
```

## Running the Container

To start a container with the Python environment, run:

```bash
docker run -it --rm -p 8000:8000 -v $(pwd):/home/python-user/work python-dev bash
```

This will:

- Start a container with the Python development image
- Map port 8000 from the container to your host (for web applications)
- Mount your current directory to the work directory in the container
- Start an interactive bash session
- Remove the container when you exit the bash session

## Developing Python Applications

Once inside the container, you can:

- Run Python scripts with `python your_script.py`
- Use IPython for interactive development with `ipython`
- Run tests with `pytest`
- Format code with `black`
- Check syntax with `flake8`
- Check types with `mypy`

## Creating Virtual Environments

If you need to create a virtual environment within the container:

```bash
python -m venv venv
source venv/bin/activate
pip install -r requirements.txt  # if you have a requirements file
```

## Customization

You can customize the Dockerfile to:

- Change the Python version
- Add additional Python packages by modifying the `pip install` command
- Add additional system dependencies by modifying the `apt-get install` command
- Change the user or security settings
