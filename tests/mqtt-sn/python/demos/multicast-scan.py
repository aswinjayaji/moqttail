import socket
import struct
import sys

multicast_group = "225.1.1.1"
multicast_group ="225.0.18.83" #rsmb
multicast_port=1885
server_address = ('', multicast_port)
code="utf8"
host_address=dict()
BEGIN="HTTP/1.1"
timeout=0.25

# Create the socket
socket.setdefaulttimeout(timeout)
group = (multicast_group, multicast_port)

sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM, socket.IPPROTO_UDP)
sock.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
#sock.setsockopt(socket.IPPROTO_IP, socket.IP_MULTICAST_TTL, 2)

mreq = struct.pack('4sL', socket.inet_aton(multicast_group), socket.INADDR_ANY)
sock.setsockopt(socket.IPPROTO_IP, socket.IP_ADD_MEMBERSHIP, mreq)
sock.bind(server_address)



count =0
data_in=""
def read_data():
    try:
        response = sock.recvfrom(1024)
        return response
    except socket.timeout:
        #print("timeout")
        return False
print('\nstarting scan')  
while True:
    #print('\nwaiting to receive message')
    data=read_data()
    if data:
        print(data)









# Example:
# import ssdp
#
#ssdp.discover("roku:ecp")
