import docker
import pytest
import subprocess
import random
import paho.mqtt.client as mqtt
import logging
from time import sleep
from pathlib import Path
import socket
import struct
import sys

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

logging.info("Start logging")

def test(
    mosquitto_container, mosquitto_mqtt_port, mosquitto_username, mosquitto_password
):
    mqttc = mqtt.Client(callback_api_version=mqtt.CallbackAPIVersion.VERSION2)
    topic_name = "test123"

    @mqttc.message_callback()
    def on_message(client, userdata, message):
        print("[TEST]message received " + str(message.payload.decode("utf-8")))
        print("[TEST]message topic=" + str(message.topic))
        print("[TEST]message qos=" + str(message.qos))
        print("[TEST]message retain flag=" + str(message.retain))

    @mqttc.connect_callback()
    def on_connect(client, userdata, flags, reason_code, properties):
        print(f"[TEST] Connected with result code {reason_code}")
        # Subscribing in on_connect() means that if we lose the connection and
        # reconnect then subscriptions will be renewed.
        client.subscribe(topic_name, 0)


    @mqttc.subscribe_callback()
    def on_subscribe(client, userdata, granted_qos):
            print("[TEST] Subscribed to Topic: " +  topic_name + " with QoS: " + str(granted_qos))


    @mqttc.log_callback()
    def on_log(client,userdata,level,buff):
         print("[TEST] Log: " + str(buff))

    mqttc.on_message = on_message
    mqttc.on_connect = on_connect
    mqttc.on_subscribe = on_subscribe
    mqttc.on_log = on_log


    port = random.randint(30000, 32000)

    run_params =  [
            "target/debug/be-server",
            "--lport", f"{port}",
            "--lhost", "127.0.0.1",
            "--mhost", "127.0.0.1",
            "--mport", f"{mosquitto_mqtt_port}",
            "--muser", f"{mosquitto_username}",
            "--mpassword", f"{mosquitto_password}",
            "--mtopic", f"{topic_name}"
    ]

    print(run_params)

    pe_process = subprocess.Popen(
        run_params
    )
    sleep(3)

    mqttc.username_pw_set(mosquitto_username, mosquitto_password)
    mqttc.connect("localhost", mosquitto_mqtt_port)
    
    sleep(2)

    s = socket.socket(socket.AF_INET)

    s.connect(("127.0.0.1",port))
    print("Connected")
    mqttc.loop(timeout=2)
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
    mqttc.loop(timeout=20)
    sleep(1)
    pe_process.send_signal(2)

    assert False
