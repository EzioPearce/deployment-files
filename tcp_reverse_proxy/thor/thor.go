package main

import (
	"bufio"
	"flag"
	"fmt"
	"io"
	"log"
	"net"
	"os"
	"time"
)

func main() {
	//op := flag.String("type", "", "Server (s) or client (c) ?")
	port := flag.String("port", "5001", "address? host:port")
	flag.Parse()

	fmt.Println(*port) //prints the port number
	runClient(*port)   //runs the client
}

func runClient(port string) error {
	conn, err := net.Dial("tcp", "127.0.0.1:"+port)
	if err != nil {
		return err
	}
	defer conn.Close()

	scanner := bufio.NewScanner(os.Stdin) // write message to the proxy here
	fmt.Println("Message to be sent?")    // Input the message to be sent from client to server
	for scanner.Scan() {
		fmt.Println("Writing", scanner.Text())
		conn.Write(append(scanner.Bytes(), '\r'))

		fmt.Println("Messsage sent to server") //confirmation message
		buffer := make([]byte, 1024)
		conn.SetReadDeadline(time.Now().Add(5 * time.Second))
		_, err := conn.Read(buffer)

		if err != nil && err != io.EOF {
			log.Fatal(err)
		} else if err == io.EOF {
			log.Println("Connection is closed")
			return nil
		}

		fmt.Println(string(buffer))

	}
	return scanner.Err()
}
