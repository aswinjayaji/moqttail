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
import sys
sys.path.append('c:/python34/steve/mqttsclient/')
print(sys.version_info)

##

from MQTTSNclient import Callback
from MQTTSNclient import Client
from MQTTSNclient import publish
import MQTTSN
message_q=queue.Queue()

class MyCallback(Callback):
  def messageArrived(self,client,TopicId, payload, qos, retained, msgid):
    m= "d-p-Arrived" +" topic  " +str(TopicId)+ "message " +\
       str(payload) +"  qos= " +str(qos) +" ret= " +str(retained)\
       +"  msgid= " + str(msgid)
    #print("got the message ",payload)
    message_q.put(payload)
    return True



######
def empty_queue(delay=0):
    while not message_q.empty():
      m=message_q.get()
      print("Received message  ",m)
    if delay!=0:
      time.sleep(delay)
########

#if __name__ == "__main__":
host="192.168.1.159"
port=1885
#m_port=1885 #port gateways advertises on
#m_group="225.0.18.83" #IP gateways advertises on

client = Client("linh")#change so only needs name
client.message_arrived_flag=False
client.registerCallback(MyCallback())
"""
client.find_gateway(m_port,m_group)

while not client.GatewayFound:
  print("waiting for gateway")
  time.sleep(1)

############



"""
print ("threads ",threading.activeCount()) 
print("connecting ",host)
client.connected_flag=False

client.connect(host,port)

client.lookfor(MQTTSN.CONNACK)
try:
  if client.waitfor(MQTTSN.CONNACK)==None:
      print("connection failed")
      raise SystemExit("no Connection quitting")
except:
  print("connection failed")
  raise SystemExit("no Connection quitting")
  
client.loop_start() #start loop
print ("threads ",threading.activeCount()) 
topic1="mqttsn-test"
print("topic for topic1 is", topic1)
print("connected now subscribing")
#register topic
topic2="mqtt-test"
topic2_id=client.register(topic2)
client.subscribed_flag=False
while True:
  rc, topic1_id = client.subscribe(topic1,1)
  if rc==None:
    print("subscribe failed")
    raise SystemExit("Subscription failed quitting")
  if rc==0:
    print("subscribed ok to ",topic1)
    break
count=0
msg="test message from MQTTSN"
try:
  while True:
    nmsg=msg+str(count)
    count+=1
    print("publishing message ",nmsg)
    id= client.publish(topic2_id,nmsg,qos=0)
    time.sleep(1)
    empty_queue(0)
    pass
except KeyboardInterrupt:
    print ("You hit control-c")



print ("threads ",threading.activeCount()) 

print("disconnecting")
client.disconnect()

client.loop_stop()
