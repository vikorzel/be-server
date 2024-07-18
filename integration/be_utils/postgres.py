"""Utils to work with postgres"""

import json
from time import time
from time import sleep as time_sleep
import psycopg

TABNAME = "DeviceConfig"


def _init_table(connection: psycopg.Connection):
    cursor = connection.cursor()

    cursor.execute(
        """
SELECT tablename
FROM pg_catalog.pg_tables
WHERE schemaname != 'pg_catalog' AND 
schemaname != 'information_schema';"""
    )
    for row in cursor:
        if row[0] == "TABNAME":
            return
    cursor.execute(
        f"""CREATE TABLE {TABNAME} (
ID INT PRIMARY KEY,
type VARCHAR(255),
config JSONB,
ts NUMERIC 
    )
"""
    )
    connection.commit()
    return


def set_frige_config(connection: psycopg.Connection, device_id: int, config: dict):
    """Set config for frige"""
    try:
        _init_table(connection)
    except psycopg.errors.DuplicateTable:
        pass
    attempt = 0
    while True:
        if attempt > 3:
            raise Exception("Can't write to DB")
        try:
            attempt += 1
            cur = connection.cursor()
            cur.execute(
                f"""INSERT INTO {TABNAME} (ID, type, config, ts) VALUES (%s, %s, %s, %s)""",
                (
                    device_id,
                    "fridge",
                    json.dumps(config),
                    int(time()),
                ),
            )
            connection.commit()
            time_sleep(0.05)
            break
        except psycopg.errors.InFailedSqlTransaction as e:
            print(e)
            continue
    return
