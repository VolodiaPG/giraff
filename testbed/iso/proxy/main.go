// Package implements a simple HTTP proxy that adds a `Proxy-Timestamp` in the request header with the timestamp the request is received here.
package main

import (
	"fmt"
	"io"
	"log"
	"net/http"
	"os"
	"strconv"
	"time"
)

var customTransport = http.DefaultTransport

func init() {
	// Here, you can customize the transport, e.g., set timeouts or enable/disable keep-alive
}

func handleRequest(w http.ResponseWriter, r *http.Request) {
	// Create a new HTTP request with the same method, URL, and body as the original request
	targetURL := r.URL
	proxyReq, err := http.NewRequest(r.Method, targetURL.String(), r.Body)
	if err != nil {
		http.Error(w, "Error creating proxy request", http.StatusInternalServerError)
		return
	}

	proxyReq.Header = r.Header
	proxyReq.Header.Add("Proxy-Timestamp", strconv.FormatInt(time.Now().UnixMilli(), 10))
	proxyReq.ContentLength = r.ContentLength

	// // Copy the headers from the original request to the proxy request
	// for name, values := range r.Header {
	// 	for _, value := range values {
	// 		proxyReq.Header.Add(name, value)
	// 	}
	// }

	// Send the proxy request using the custom transport
	resp, err := customTransport.RoundTrip(proxyReq)
	if err != nil {
		http.Error(w, "Error sending proxy request", http.StatusInternalServerError)
		fmt.Println("Error sending proxy request", err)
		return
	}
	// tUnixMilliResp := int64(time.Nanosecond) * time.Now().UnixNano() / int64(time.Millisecond)
	// w.Header().Add("Proxy-Timestamp", strconv.FormatInt(tUnixMilliResp, 10))
	defer resp.Body.Close()

	// Copy the headers from the proxy response to the original response
	for name, values := range resp.Header {
		for _, value := range values {
			w.Header().Add(name, value)
		}
	}

	// Set the status code of the original response to the status code of the proxy response
	w.WriteHeader(resp.StatusCode)

	// Copy the body of the proxy response to the original response
	io.Copy(w, resp.Body)
}

func main() {
	port := os.Getenv("PORT")
	// Create a new HTTP server with the handleRequest function as the handler
	server := http.Server{
		Addr:    ":" + port,
		Handler: http.HandlerFunc(handleRequest),
	}

	// Start the server and log any errors
	log.Println("Starting proxy server on", port)
	err := server.ListenAndServe()
	if err != nil {
		log.Fatal("Error starting proxy server: ", err)
	}
}
