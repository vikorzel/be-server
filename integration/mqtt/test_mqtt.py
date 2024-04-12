'''Tests for MQTT functional'''
import subprocess
import random
import logging
from time import sleep
from pathlib import Path
import socket
import struct
import sys
import threading
import json
from queue import Queue
import paho.mqtt.subscribe as mqsub
import paho.mqtt.client as mqclient
from paho.mqtt.enums import MQTTProtocolVersion
import be_utils.be_server as be_server_helper #pylint: disable=E0401
import pytest

test_data_path = (
    Path(__file__).parent.parent.parent.absolute().as_posix() + "/test_data"
)

stdout_handler = logging.StreamHandler(stream=sys.stdout)
file_handler = logging.FileHandler(filename='tmp.log')
file_handler.setLevel(logging.DEBUG)

logging.basicConfig(
     level=logging.DEBUG,
     format='%(asctime)s - %(name)s : %(levelname)s - %(message)s',
     handlers=[stdout_handler, file_handler])


def test(
    mosquitto_container, mosquitto_mqtt_port, mosquitto_username, mosquitto_password, be_service_port
):
    '''Testing that BE server works properly with MQTT'''
    topic_name = f"random_topic_{random.randint(1,1000)}"

    def start_subscription(queue:Queue):
        print("[TEST] Setup callback")


        def on_message(client: mqclient.Client , udata, message):
            print(f"[TEST] {message.topic}: {message.payload}")
            queue.put(message)
            if queue.full():
                client.disconnect()

        mqsub.callback(
            callback=on_message,
            topics=[topic_name],
            port=mosquitto_mqtt_port,
            auth={"username": mosquitto_username, "password": mosquitto_password},
            protocol=MQTTProtocolVersion.MQTTv311,
            keepalive=5,
            client_id="test"
        )

    queue = Queue(2)
    mqtt_thread = threading.Thread(target=start_subscription, args=[queue])
    mqtt_thread.start()



    port = random.randint(30000, 32000)

    run_params =  [
            "target/debug/be-server",
            "--lport", f"{port}",
            "--lhost", "127.0.0.1",
            "--mhost", "127.0.0.1",
            "--mport", f"{mosquitto_mqtt_port}",
            "--muser", f"{mosquitto_username}",
            "--mpassword", f"{mosquitto_password}",
            "--mtopic", f"{topic_name}",
            "--sport", f"{be_service_port}"
    ]

    print(run_params)

    pe_process = subprocess.Popen(
        run_params
    )
    be_server_helper.wait_till_service_start(be_service_port, 10)


    s = socket.socket(socket.AF_INET)

    s.connect(("127.0.0.1",port))
    temperature = bytearray(struct.pack("f", 0.32))
    humidity = bytearray(struct.pack("f", 0.123))
    buf = []
    buf.append(12)
    buf.append(2)


    temperature = struct.pack("f", 0.32)
    humidity = struct.pack("f", 0.123)


    buf += temperature
    buf += humidity

    temperature = struct.pack("f", 0.43)
    humidity = struct.pack("f", 0.98)
    buf += temperature
    buf += humidity
    s.send(bytearray(buf))

    sleep(0.1)
    print("From queue:")
    devices = []
    while not queue.empty():
        devices.append(queue.get(timeout=2).payload)
    print(devices)
    pe_process.send_signal(2)

    elements = [{
        'id': 1201,
        'name': "Device 1201",
        'temperature': 0.32,
        "humidity": 0.123
    },
    {
        'id': 1202,
        'name': "Device 1202",
        'temperature': 0.43,
        'humidity': 0.98
    }]

    devices_objects = [json.loads(device) for device in devices]
    assert len(devices_objects) == len(elements)
    elements.sort(key = lambda x: x["id"])
    devices_objects.sort(key = lambda x: x["id"])
    actual_str = [json.dumps(x) for x in devices_objects ]
    expect_str = [json.dumps(x) for x in elements ]
    assert all([a == b for a, b in zip(expect_str, actual_str)]), "received via MQTT exactly what sent"