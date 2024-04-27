import pytest
import psycopg
from be_utils import postgres

def test(postgres_container, postgress_connection_string):
    connection = psycopg.connect(postgress_connection_string)
    postgres.set_frige_config(connection, 1234, {'temperature': 3, 'humidity': 10})
    cur = connection.cursor()
    cur.execute(f"SELECT * from {postgres.TABNAME}")
    print(cur.fetchall())
    assert False