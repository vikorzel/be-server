'''Fixtures for work with mosquitto'''

# pylint:disable=redefined-outer-name

import os
import subprocess
import time
from pathlib import Path
import tempfile
import docker
import be_utils.common as common_utils # pylint: disable=E0401
import pytest

get_free_port = common_utils.get_free_port

@pytest.fixture(scope="session")
def mosquitto_data_path(data_path):
    """path to local mosquitto data folder"""
    return Path(data_path, "mosquitto")


@pytest.fixture(scope="session")
def mosquitto_container_name():
    """name of the mosquitto container image"""
    return "eclipse-mosquitto"


@pytest.fixture(scope="session")
def mosquitto_username():
    """mosquitto username for tests"""
    return "testuser"


@pytest.fixture(scope="session")
def mosquitto_password():
    """mosquitto password for tests"""
    return common_utils.generate_password()


@pytest.fixture(scope="module")
def mosquitto_secrets_path(mosquitto_data_path):
    """path to local mosquitto secrets folder"""
    return Path(mosquitto_data_path, "secrets")


@pytest.fixture(scope="module")
def mosquitto_logs_path(mosquitto_data_path):
    """path to local mosquitto logs folder"""
    return Path(mosquitto_data_path, "logs")


@pytest.fixture(scope="module")
def mosquitto_config_folder_path(mosquitto_data_path):
    """path to local mosquitto config folder"""
    return Path(mosquitto_data_path, "config")


@pytest.fixture(scope="module")
def mosquitto_config():
    """mosquitto config file content for tests"""
    return [
        "connection_messages true",
        "log_type all",
        "listener 1883",
        "persistence true",
        "persistence_location /mosquitto/data/",
        "log_dest file /mosquitto/log/mosquitto.log",
        "password_file /mosquitto/secrets/passfile.txt",
    ]


@pytest.fixture(scope="module")
def mosquitto_config_path(mosquitto_config_folder_path, mosquitto_config):
    """path to local mosquitto config file for tests"""
    config_path = Path(mosquitto_config_folder_path, "mosquitto.conf")
    if not config_path.is_file():
        with open(config_path, "w", encoding="utf-8") as config_file:
            config_file.write("\n".join(mosquitto_config))
    return config_path


@pytest.fixture(scope="module")
def mosquitto_log_path(mosquitto_data_path):
    """path to local mosquitto log folder for tests"""
    return Path(mosquitto_data_path, "log")


@pytest.fixture(scope="module")
def mosquitto_storage_path():
    """path to local mosquitto data folder for tests"""
    return Path(tempfile.gettempdir())


@pytest.fixture(scope="module")
def mosquitto_mqtt_port():
    """port to use for mosquitto mqtt server for tests"""
    return get_free_port()


@pytest.fixture(scope="module", autouse=True)
def mosquitto_password_init(
    mosquitto_secrets_path,
    mosquitto_container_name,
    mosquitto_username,
    mosquitto_password,
):
    """initialize mosquitto password file for tests"""
    passfile = Path(mosquitto_secrets_path, "passfile.txt")
    if passfile.is_file():
        return
    client = docker.from_env()
    client.containers.run(
        image=mosquitto_container_name,
        command=f"mosquitto_passwd -b -c /secrets/passfile.txt {mosquitto_username} {mosquitto_password}", #pylint:disable=line-too-long
        volumes=[f"{mosquitto_secrets_path}:/secrets/"],
    )


@pytest.fixture(scope="module")
def mosquitto_container(
    mosquitto_container_name,
    mosquitto_config_path,
    mosquitto_storage_path,
    mosquitto_log_path,
    mosquitto_mqtt_port,
    mosquitto_secrets_path
):
    """start mosquitto container for tests"""
    client = docker.from_env()
    container = client.containers.run(
        mosquitto_container_name,
        detach=True,
        volumes=[
            f"{mosquitto_config_path}:/mosquitto/config/mosquitto.conf",
            f"{mosquitto_storage_path}:/mosquitto/data",
            f"{mosquitto_log_path}:/mosquitto/log",
            f"{mosquitto_secrets_path}:/mosquitto/secrets"
        ],
        ports={"1883/tcp": mosquitto_mqtt_port},
        user = os.getuid()
    )
    process = subprocess.Popen(['tail', '-f', f'{mosquitto_log_path}/mosquitto.log'],  stdout=subprocess.PIPE,stderr=subprocess.PIPE)
    start_time = time.time()
    while True:
        output = process.stdout.readline()
        if b"Opening ipv4 listen socket on port 1883" in output:
            break
        if time.time() - start_time > 10:
            raise TimeoutError("Timeout waiting for mosquitto to start")
    time.sleep(2)
    yield container
    container.stop()