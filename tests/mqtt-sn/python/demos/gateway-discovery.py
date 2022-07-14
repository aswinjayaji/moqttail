"""
/*******************************************************************************
 * Copyright (c) 2011, 2013 IBM Corp.
 *
 * All rights reserved. This program and the accompanying materials
 * are made available under the terms of the Eclipse Public License v1.0
 * and Eclipse Distribution License v1.0 which accompany this distribution. 
 *
 * The Eclipse Public License is available at 
 *    http://www.eclipse.org/legal/epl-v10.html
 * and the Eclipse Distribution License is available at 
 *   http://www.eclipse.org/org/documents/edl-v10.php.
 *
 * Contributors:
 *    Ian Craggs - initial API and implementation and/or initial documentation
 *******************************************************************************/
"""
#add by me
import queue
import struct
import threading
import time
import signal
import sys
sys.path.append('c:/python34/steve/mqttsclient/')
print(sys.version_info)
##

from MQTTSNclient import Callback
from MQTTSNclient import Client
from MQTTSNclient import publish
import MQTTSN
message_q=queue.Queue()
gateways=[]
def keyboardInterruptHandler(signal, frame):
    print("KeyboardInterrupt (ID: {}) has been caught. Cleaning up...".format(signal))
    exit(0)
class MyCallback(Callback):
  def advertise(self,client,address, gwid, duration):
    m="advertise -address" + str(address)+"qwid= " +str(gwid)+"dur= " +str(duration)
    print ("found gateway at",m)
    ip_address,port=address
    temp=[ip_address,port,str(gwid),str(duration)]
    gateways.append(temp)
    client.GatewayFound=True

######
def empty_queue(delay=0):
    while not message_q.empty():
      m=message_q.get()
      print(m)
    if delay!=0:
      time.sleep(delay)
########
signal.signal(signal.SIGINT, keyboardInterruptHandler)
print ("threads ",threading.activeCount()) 
#if __name__ == "__main__":
host="192.168.1.159"
port=1883
s_port=1885
s_group="224.0.18.83"
r_port=1885 #port gateways advertises on
r_group="225.0.18.83" #IP gateways advertises on
#r_port=1883 #port gateways advertises on
#r_group="225.1.1.1" #IP gateways advertises on
client = Client("linh")#change so only needs name
print ("threads ",threading.activeCount()) 
client.registerCallback(MyCallback())

client.find_gateway(r_port,r_group)
client.loop_start_gw()
while not client.GatewayFound:
#while True:
    print ("waiting for gateway threads ",threading.activeCount()) 
    time.sleep(10)



############

print("Gateway found")
gateway=gateways.pop(0)
host=gateway[0]
port=gateway[1]

host="192.168.1.159"
port=1885
print ("threads ",threading.activeCount()) 
print("connecting ",host,"  ",port)
client.connected_flag=False

client.connect(host,port)

client.lookfor(MQTTSN.CONNACK)
if client.waitfor(MQTTSN.CONNACK)==None:
    print("connection failed")
    raise SystemExit("no Connection quitting")


time.sleep(2)
topic1="test"
topic1_id=client.register(topic1)
print("topic id for topic1 is", topic1_id)
empty_queue(4)
msg="aaaaa"

for i in range(5):
    msg1=msg+str(i)
    print("publishing message ",msg1, "topic ",topic1_id)
    id= client.publish(topic1_id,msg1,qos=1)
    print("published id = ",id)
    time.sleep(2)
    while not message_q.empty():
        m=message_q.get()
        print(m)


print("disconnecting")
client.disconnect()
time.sleep(10)
print ("threads ",threading.activeCount()) 
empty_queue(4)
client.loop_stop()



