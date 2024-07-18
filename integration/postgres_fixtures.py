"""Fixtures for work with PostgreSQL"""

# pylint:disable=redefined-outer-name
import docker
import os
import faker
import psycopg
import be_utils.common as common_utils  # pylint: disable=E0401
import pytest
import time


@pytest.fixture(scope="session")
def postgress_password():
    """Password of postgress user"""
    return common_utils.generate_password(20)


@pytest.fixture(scope="session")
def postgress_image_name():
    """Name of postgress image"""
    return "postgres"


@pytest.fixture(scope="session")
def postgress_port():
    """External port of postgress"""
    return common_utils.get_free_port()


@pytest.fixture(scope="session")
def postgres_username():
    """Username of postgres user"""
    return faker.Faker().first_name()


@pytest.fixture(scope="session")
def postgress_database_name():
    """Postgress DB Name"""
    return faker.Faker().first_name()


@pytest.fixture(scope="session")
def postgres_connection_string(
    postgress_password, postgres_username, postgress_database_name, postgress_port
):
    return f"host=localhost port={postgress_port} dbname={postgress_database_name} connect_timeout=30 user={postgres_username} password={postgress_password}"


@pytest.fixture(scope="session")
def postgres_container(
    postgress_password,
    postgres_username,
    postgress_image_name,
    postgress_database_name,
    postgress_port,
    postgres_connection_string,
):
    """Returns postgress container"""
    client = docker.from_env()
    contatiner = client.containers.run(
        postgress_image_name,
        detach=True,
        ports={"5432/tcp": postgress_port},
        environment={
            "POSTGRES_USER": postgres_username,
            "POSTGRES_DB": postgress_database_name,
            "POSTGRES_PASSWORD": postgress_password
        },
    )
    wait = True
    now = time.time()
    while wait:
        try:
            connection = psycopg.connect(postgres_connection_string)
            wait = False
            connection.close()
        except psycopg.OperationalError as exc:
            if time.time() - now > 20:
                print(contatiner.logs())
                raise TimeoutError("Too long waiting psql container") from exc
    yield contatiner
    contatiner.stop()
