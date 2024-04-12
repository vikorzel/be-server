'''Utils to work with be server'''
import datetime
import requests


def wait_till_service_start(service_port: int, timeout: float = 10):
    '''Wait till BE Server will be ready'''
    start_time = datetime.datetime.now()
    while datetime.datetime.now() - start_time < datetime.timedelta(seconds=timeout):
        try:
            response = requests.get(f"http://localhost:{service_port}/status", timeout=1)
            if response.text == "OK":
                return
        except requests.exceptions.RequestException:
            pass
    raise TimeoutError("Timeout")
