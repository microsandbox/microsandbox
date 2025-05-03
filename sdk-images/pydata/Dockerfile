FROM python:3.11-slim

# Set environment variables
ENV PYTHONUNBUFFERED=1 \
    PYTHONDONTWRITEBYTECODE=1 \
    PIP_NO_CACHE_DIR=1 \
    DEBIAN_FRONTEND=noninteractive

# Install system dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    curl \
    git \
    wget \
    fonts-dejavu \
    libssl-dev \
    ca-certificates \
    && apt-get clean && rm -rf /var/lib/apt/lists/*

# Create a non-root user to run code
ARG USER_NAME="pydata-user"
ARG USER_UID="1000"
ARG USER_GID="100"

# Create non-root user and necessary directories
RUN useradd -m -s /bin/bash -N -u $USER_UID $USER_NAME \
    && mkdir -p /home/$USER_NAME/work \
    && chown -R $USER_NAME:$USER_GID /home/$USER_NAME

# Install core Python packages
RUN pip install --no-cache-dir --upgrade pip setuptools wheel \
    && pip install --no-cache-dir \
    jupyterlab \
    notebook \
    ipywidgets \
    numpy \
    pandas \
    matplotlib \
    seaborn \
    scikit-learn \
    scipy \
    statsmodels \
    plotly \
    bokeh \
    dash \
    xgboost \
    lightgbm \
    nltk \
    spacy \
    gensim \
    tensorflow \
    pytorch-cpu \
    black \
    flake8 \
    mypy \
    pytest \
    ipython

# Add Jupyter configuration
USER $USER_NAME
WORKDIR /home/$USER_NAME/work

RUN mkdir -p /home/$USER_NAME/.jupyter \
    && echo "c.NotebookApp.token = ''" > /home/$USER_NAME/.jupyter/jupyter_notebook_config.py \
    && echo "c.NotebookApp.password = ''" >> /home/$USER_NAME/.jupyter/jupyter_notebook_config.py \
    && echo "c.NotebookApp.ip = '0.0.0.0'" >> /home/$USER_NAME/.jupyter/jupyter_notebook_config.py \
    && echo "c.NotebookApp.allow_root = False" >> /home/$USER_NAME/.jupyter/jupyter_notebook_config.py \
    && echo "c.NotebookApp.open_browser = False" >> /home/$USER_NAME/.jupyter/jupyter_notebook_config.py \
    && echo "c.NotebookApp.notebook_dir = '/home/$USER_NAME/work'" >> /home/$USER_NAME/.jupyter/jupyter_notebook_config.py

# Expose the port Jupyter will run on
EXPOSE 8888

# Launch Jupyter Notebook
CMD ["jupyter", "lab", "--no-browser", "--ip=0.0.0.0", "--port=8888"]
