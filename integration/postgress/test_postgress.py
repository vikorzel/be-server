import random
import socket
import struct
import subprocess
import psycopg
from be_utils import postgres
import be_utils.be_server as be_server_helper #pylint: disable=E0401
import pytest

def test(
        # POSTGRES
        postgres_container,
        postgres_connection_string,
        postgres_username,
        postgress_password,
        postgress_port,
        postgress_database_name,

        # MQTT
        mosquitto_container,
        mosquitto_mqtt_port,
        mosquitto_username,
        mosquitto_password,
        
        # BE
        be_service_port):
    #   SETUP
    connection = psycopg.connect(postgres_connection_string, autocommit=True)
    port = random.randint(30000, 32000)
    run_params =  [
            "target/debug/be-server",
            "--lport", f"{port}",
            "--lhost", "127.0.0.1",
            "--mhost", "127.0.0.1",
            "--mport", f"{mosquitto_mqtt_port}",
            "--muser", f"{mosquitto_username}",
            "--mpassword", f"{mosquitto_password}",
            "--mtopic", f"topic.1",
            "--sport", f"{be_service_port}",
            "--plogin", f"{postgres_username}",
            "--ppassword", f"{postgress_password}",
            "--pport", f"{postgress_port}",
            "--phost", "127.0.0.1",
            "--pdbname", f"{postgress_database_name}"

    ]
    pe_process = subprocess.Popen(
        run_params
    )
    be_server_helper.wait_till_service_start(be_service_port, 10)
    #   END SETUP
    postgres.set_frige_config(connection, 101, {'temperature': 3, 'humidity': 10})
    postgres.set_frige_config(connection, 102, {'temperature': 4, 'humidity': 11})
    buf = []
    buf.append(1)
    buf.append(2)
    buf += struct.pack("f", random.random())
    buf += struct.pack("f", random.random())
    buf += struct.pack("f", random.random())
    buf += struct.pack("f", random.random())



    s = socket.socket(socket.AF_INET)
    s.connect(("127.0.0.1", port))
    s.send(bytearray(buf))
    rcv = s.recv(1024)
    
    id1 = struct.unpack("i", rcv[:4])[0]
    humidity1 = struct.unpack("f", rcv[4:8])[0]
    temperature1 = struct.unpack("f", rcv[8:12])[0]

    rcv = s.recv(1024)
    id2 = struct.unpack("i", rcv[:4])[0]
    humidity2 = struct.unpack("f", rcv[4:8])[0]
    temperature2 = struct.unpack("f", rcv[8:12])[0]


    s.close()
    pe_process.send_signal(2)    
    assert id1 == 101
    assert temperature1 == 3
    assert humidity1 == 10

    assert id2 == 102
    assert temperature2 == 4
    assert humidity2 == 11