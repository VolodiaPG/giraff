# EnosLib

Enoslib is a library that enables the use of Grid'5000 for multiple purposes:
- Deploy a live environment for experimenting in `live`

## Installation
1. Install poetry
2. Install the packages
    ```shell
    poetry install
    ```
3. Enter the venv
    ```shell
    poetry run python3 integration.py
    ```

## Setup (automatically done)
1. consider moving your main SSH key under the filename `id_rsa.pub`, `ln -s ~/.ssh/id_ed25519.pub ~/.ssh/id_rsa.pub`
2. Launch and everything should work
