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

import queue
import threading
import time
import sys
#sys.path.append('c:/python34/steve/mqttsclient/')
print(sys.version_info)
from MQTTSNclient import Callback
from MQTTSNclient import Client
#from MQTTSNclient import publish
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

# host="spr.io"

host="127.0.0.1"
host="10.5.5.9"
host="10.0.2.26"

if (len(sys.argv)>1):
      host=sys.argv[1]
else:
      print("host ip not given on command line")
      print("Try: python mqtt-sn-rust-client.py host_ip ")
      sys.exit(0)
      
port=1885
port=60000
client = Client("linh")#change so only needs name
client.message_arrived_flag=False
client.registerCallback(MyCallback())
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

"""

while not client.connected_flag:
  time.sleep(1)
  print("waiting for connection")

"""
#quit()
  

print ("threads ",threading.activeCount()) 
topic1="bulbs1"
print("topic for topic1 is", topic1)
print("connected now subscribing")
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
msg="test message"

try:
  while True:
    nmsg=msg+str(count)
    count+=1
    print("publishing message ",nmsg)
    id= client.publish(topic1_id,nmsg,qos=1)
    time.sleep(1)
    empty_queue(0)
    pass
except KeyboardInterrupt:
    print ("You hit control-c")

print ("threads ",threading.activeCount()) 

print("disconnecting")
client.disconnect()
client.loop_stop()
