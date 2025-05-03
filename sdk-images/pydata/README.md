# PyData Docker Image

This directory contains a Dockerfile for setting up a Python data analysis environment with Jupyter Lab and common data science libraries.

## Features

- Python 3.11
- Jupyter Lab and Notebook
- Common data science libraries:
  - NumPy and SciPy for numerical computing
  - Pandas for data manipulation
  - Matplotlib, Seaborn, Plotly, and Bokeh for visualization
  - Scikit-learn for machine learning
  - TensorFlow and PyTorch for deep learning
  - NLTK, spaCy, and Gensim for NLP
  - XGBoost and LightGBM for gradient boosting
- Development tools (black, flake8, mypy, pytest)
- Non-root user for better security

## Building the Image

To build the Docker image, run the following command from this directory:

```bash
docker build -t pydata-dev .
```

## Running the Container

To start a Jupyter Lab server, run:

```bash
docker run -it --rm -p 8888:8888 -v $(pwd):/home/pydata-user/work pydata-dev
```

This will:

- Start a container with the PyData image
- Map port 8888 from the container to your host
- Mount your current directory to the work directory in the container
- Remove the container when you stop it

For a bash terminal instead of Jupyter:

```bash
docker run -it --rm -v $(pwd):/home/pydata-user/work pydata-dev bash
```

## Accessing Jupyter Lab

Once the container is running, you can access Jupyter Lab by opening a browser and navigating to:

```
http://localhost:8888
```

No password or token is required (for simplicity in local development).

## Data Analysis Workflow

With this PyData environment, you can:

- Create and run Jupyter notebooks
- Load and manipulate data using Pandas
- Perform statistical analysis with NumPy, SciPy, and StatsModels
- Create visualizations with Matplotlib, Seaborn, Plotly, or Bokeh
- Build machine learning models with Scikit-learn, XGBoost, or LightGBM
- Develop deep learning models with TensorFlow or PyTorch
- Process text data with NLTK, spaCy, or Gensim
- Create interactive dashboards with Dash

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

## Security Note

For production use, it's recommended to set a password or token in the Jupyter configuration.
