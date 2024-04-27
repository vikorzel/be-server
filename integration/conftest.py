"""Common pytests fixtures"""

# pylint:disable=redefined-outer-name


from pathlib import Path
import pytest
import be_utils.common as common_utils # pylint: disable=E0401

pytest_plugins = ['mosquito_fixtures', 'postgres_fixtures']



@pytest.fixture(scope="session")
def data_path():
    """path to tests data folder"""
    return Path(Path(__file__).parent.parent, "test_data")


@pytest.fixture(scope="session")
def be_service_port():
    """return avaliable port for BE Service"""
    return common_utils.get_free_port()
