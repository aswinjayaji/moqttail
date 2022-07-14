import queue
import struct
import threading
import time
import sys
sys.path.append('c:/python34/steve/mqttsclient/')
from MQTTSNclient import Callback
from MQTTSNclient import Client
from MQTTSNclient import publish
import MQTTSN


#client = Client("linh_pub")#change so only needs name

print ("threads ",threading.activeCount()) 

for i in range(10):
    publish("tt","test",port=1885,host="192.168.1.159")
time.sleep(3)


