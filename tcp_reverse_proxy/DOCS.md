# **TCP PROXY**:
This project demonstrates the working of a raw configurable TCP proxy that listens on multiple ports as requested. 
    The Proxy Server, named **`Odin`** is written in Rust and the TCP Client to test it , named **`Thor`** is written in Golang.

## **WHAT IS A TCP PROXY?**
A proxy acts a _gateway_ between the client and the internet. It is located between the client and the server and routes all traffic from the client to the server. It then collects the response from the server and transfers the requested information back to the client.
![TCP-Proxy](public/img/tcp%20proxy.drawio.png)
    
In this case, Odin is an intermediary server separating client requests from the different backend hosts. Traffic flows through the proxy server on it's way to the requested address.


## **WHAT HAS BEEN BUILT AND HOW IT WORKS?**
This project demonstrates the working of a simple tcp proxy which supports the mentioned config backend server and listens for connections on the mentioned ports. 
The Thor client sends out connection requests to the specified ports to access the backend.
As mentioned, the odd number requests returns the request message as it is, while the  even number of connections return the message in Uppercase.
Odin proxies the incoming connection from Thor and routes it to the appropriate target server in the backend. 

        Running the Code:
            Step 1: Build and start Odin:

                cargo build
                cargo run -- -f config.json.
               
            
            Step 2 : Build and run Thor:
                
                go build thor.go
                ./thor -port <port number to send the request on>
                
            Step 3: Input the message

    
## **HOW DID I LAND AT THIS DESIGN?**
After spending 3 days trying to study and understand the techstack driving the core functionality at fly.io, I have decided to implement an elegant solution which adheres to the techstack used at Fly. 

Keeping the aforementioned point in mind, the proxy has been implemented in rust and the test-client has been set up in Go.

Initial focus was on setting up a proxy. Basic skeletal design was implemented using static `localhost` address listening on a single port. Once this step was achieved, the focus was then to support the config file given for the challenge. 
Keeping that in mind, I have decided to pass the config file as a command line argument during runtime, where the code parses the file. The supported formats for the config file include `JSON, YAML and TOML`. 

At this point , standard rust libraries were being used and it had occurred that concurrency was not being handled efficiently. It was then that the code was refactored to work with `tokio` library to handle connections _asynchronously_ at runtime using the `spawn()` method. This solved the problem of concurrency in order to support multiple connection requests in an efficient manner. 

Once concurrency had been handled, the next step was to have the proxy listen on multiple ports for incoming connections and route then to appropriate targets. This required the implementation of basic **load balancing** and hence **Round Robin** algorithm has been selected for this purpose.
The proxy has been configured for balancing the incoming load between multiple targets. the incoming requests are routed to the different targets based on the port request of the incoming connection. 
Round Robin way is a simple effective way to distribute client requests across a group of servers. A client request is forwarded to every server in an incremental order. The algorithm instructs the load balancer to go back to the top of the list and repeat the process. It implements a _cyclic_ method to route and forward incoming requests.
`RoundRobinBackend` is the structure that has been written in order to implement Round-Robin load balancing for the Backend.

![Load-Balancing](public/img/load%20balancing.drawio-3.png)

In order to improve the efficiency of the proxy, the dynamic backend has been implemented with a shared state concurrency.
This means that multiple threads cannot access the same data at the same time. The connections are mutually exclusive of one another but in the `run_app()` function, it is seen that the dynamic backend automatically references the targets assigned by the round robin algorithm and these processes occur mutually exclusive of one another.

Finally, the `handle_connection()` function is used to configure the proxy for incoming connections to the server from the client. It handles the requests concurrently using the tokio spawn() method and proxies the incoming requests from client to the target backend server.

As for the client (Thor), the main priority was to generate user traffic and pass messages to the backend server via the proxy. A simple Golang program has been implemented for this purpose.
Thor is a TCP Client that sends out a message to the backend server via the Odin Proxy.
This basic piece of code just ensures that the Odin proxy is working as intended with the configured backend.

## **WHAT MIGHT BREAK UNDER A PRODUCTION LOAD?**
Although this is a fully functional TCP proxy, it does have its shortcomings
- Odin requires a fully functional backend. Error Handling and target healthchecks on the Backend has not been implemented yet. 
- Odin implements Round Robin Load balancing. In a case where the backend fails, Odin continues to forward connection requests to the unhealthy target which is problematic especially at production because the client is not guarenteed of a response all the time. 

## **WHAT NEEDS TO BE IMPLEMENTED BEFORE THE PROXY IS PRODUCTION READY?**

- Target health check must be configured. This would enable Odin to facilitate efficient load balancing to healty targets.

- Data encryption can be done in transit and enable role based access control

- Enable sticky connections by implementing IP Hashing.

- Depending on the functionality and implementation of the backend, Odin can have several functionalities implemented to better serve the backend

- Implementation of Odin's observability would serve paramount importance for deployment of any service to production. Integrating end to end observability metrics which include (But not limited to) the following:
            
    - Performance monitoring which include resource utilization
    - Create alerts for unhealthy targets.
    - import metrics for testing. 
        
The above observability metrics can be brought into light with the help of Prometheus or Grafana. These services provide visualization of the production environment at any given point in time and will aid in definitive Root Cause Analysis.
        
    
## **IF I WERE TO START ALL OVER AGAIN, WHAT WOULD BE CHANGED?**

Given a chance to redesign and implement this project, One of the functions that can be done differently would be the Round Robin algorithm for load balancing. 
Instead of the regular Round Robin Algorithm, Weighted Least connection protocol could be used where the load balancing is done by forwarding the connections to targets based on the number of existing connections and the load on each of them. 

With the above idea, different targets with different capacity can be used to handle connections of different magnitude. 
Connections are set up holistically in this menner and with the concurrency provided by tokio, the proxy worksseamlessly even under healvy loads.

One other thing that could be implemented would definitely be the target healthchecks. With the Weighted Least connection protocol and the target health checks, Odin would be completely be production ready with flexible configurable backend and healthy targeting routing.

An additional feature would be to list the connections, timestamped along with the duration of each connection. This would help in improving the overall observability metrics.
    
## MAKING A GLOBAL CLUSTERED VERSION OF THE PROXY:
    
Creating a global clustered version of the proxy enables organizations to handle heavy incoming traffic and optimizes load balancing to provide the best user experience with miniaml latency.

There are multiple ways to set up a global clustered version of the proxy.
1. **External TCP Proxy Load Balancing** :
        
External TCP Proxy Load Balancing is a reverse proxy load balancer that distributes TCP traffic coming from the internet to virtual machine (VM) instances on the organization data centers. When using External TCP Proxy Load Balancing, traffic coming over a TCP connection is terminated at the load balancing layer, and then forwarded to the closest available backend using TCP.

External TCP Proxy Load Balancing lets our architecture use a single IP address for all users worldwide. The TCP proxy load balancer automatically routes traffic to the backends that are closest to the user. This method is usually intended for TCP traffic on specific well-known ports

![External-Load-Balancing](public/img/External%20LB.drawio-2.png)

2. **Using the Anycast Protocol**:

Anycast is a network addressing and routing protocol in which the incoming connections can be routed to a variety of different locations [also called nodes]

The protocol can be set to route the incoming connection requests to the nearest data center.
- The Odin proxy can be configured globally with the backend on multiple regions
- A central anycast server must be set up that routes incoming connections to the nearest data center.

Pros:
- Depending on the incoming traffic, resources at different regions can be either scaled up or down.
- Resource utilization can be optimized based on the traffic at a particular region
- Enhanced user experience as the incoming connection requests are routed to the nearest Proxy server with load balancing therefore the latency is minimal.

![Anycast setup](public/img/any%20cast.drawio-3.png)
