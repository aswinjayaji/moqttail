import queue
import time
import sys
import paho.mqtt.client as paho
print(sys.version_info)

broker="192.168.1.159"

message_q=queue.Queue()

def empty_queue(delay=0):
    while not message_q.empty():
      m=message_q.get()
      print("Received message  ",m)
    if delay!=0:
      time.sleep(delay)
#define callback
def on_message(client, userdata, message):
   time.sleep(1)
   print("received message =",str(message.payload.decode("utf-8")))


client= paho.Client("client-001")  #create client object client1.on_publish = on_publish                          #assign function to callback client1.connect(broker,port)                                 #establish connection client1.publish("house/bulb1","on")  
######
client.on_message=on_message
#####
topic1="mqttsn-test"
topic2="mqtt-test"
print("connecting to broker ",broker)
client.connect(broker)#connect
client.loop_start() #start loop to process received messages
print("subscribing to ",topic2)

client.subscribe(topic2)#subscribe
time.sleep(3)
count=0
msg="test from mqtt client"
try:
  while True:
    nmsg=msg+str(count)
    count+=1
    print("publishing message ",nmsg)
    id= client.publish(topic1,nmsg,qos=0)
    time.sleep(1)
    empty_queue(0)
    pass
except KeyboardInterrupt:
    print ("You hit control-c")


time.sleep(1)


client.disconnect() #disconnect
client.loop_stop() #stop loop
