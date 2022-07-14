
import queue
import struct
import threading
import time
import sys
sys.path.append('c:/python34/steve/mqttsclient/')

##

from MQTTSNclient import Callback
from MQTTSNclient import Client
from MQTTSNclient import publish
import MQTTSN
message_q=queue.Queue()

######
def empty_queue(delay=0):
    while not message_q.empty():
      m=message_q.get()
      print(m)
    if delay!=0:
      time.sleep(delay)
########


host="192.168.1.159"
port=1885
#port=10000
m_port=1885 #port gateways advertises on
m_group="225.0.18.83" #IP gateways advertises on

client = Client("linh_pub")#change so only needs name

client.registerCallback(Callback())

print ("threads ",threading.activeCount()) 
print("connecting ",host)
client.connected_flag=False

client.connect(host,port)
client.loop_start()
client.lookfor(MQTTSN.CONNACK)
try:
  if client.waitfor(MQTTSN.CONNACK)==None:
      print("connection failed")
      raise SystemExit("no Connection quitting")
except:
  print("connection failed")
  raise SystemExit("no Connection quitting")


topic1="bulbs1"
print("connected now registering topic ",topic1)


time.sleep(2)
topic1_id=client.register(topic1)

msg="aaaaa"

for i in range(20):
    nmsg=msg+str(i)
    print("publishing message ",nmsg, "topic ",topic1_id)
    id= client.publish(topic1_id,nmsg,qos=0)
    #print("published id = ",id)
    #time.sleep(1)



print("disconnecting")
client.disconnect()
time.sleep(10)
print ("threads ",threading.activeCount()) 
empty_queue(4)
client.loop_stop()

for i in range(10):
    publish("tt","test",port=1884,host="192.168.1.159")


