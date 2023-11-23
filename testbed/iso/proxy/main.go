// Package implements a simple HTTP proxy that adds a `Proxy-Timestamp` in the request header with the timestamp the request is received here.
package main

import (
	"errors"
	"fmt"
	"io"
	"log"
	"net/http"
	"os"
	"strconv"
	"time"

	influxdb2 "github.com/influxdata/influxdb-client-go/v2"
	"github.com/influxdata/influxdb-client-go/v2/api"
)

type envVars struct {
	myPort        string
	influxAddress string
	influxToken   string
	influxOrg     string
	influxBucket  string
}

func lookupVar(varName string) (string, error) {
	theVar, unset := os.LookupEnv(varName)
	if !unset {
		return "", errors.New(varName + " variable is not available in the env")
	}
	return theVar, nil
}

func initEnvVars() (envVars, error) {
	myPort, err := lookupVar("PORT")
	if err != nil {
		return envVars{}, err
	}
	influxAddress, err := lookupVar("INFLUX_ADDRESS")
	if err != nil {
		return envVars{}, err
	}
	influxToken, err := lookupVar("INFLUX_TOKEN")
	if err != nil {
		return envVars{}, err
	}
	influxOrg, err := lookupVar("INFLUX_ORG")
	if err != nil {
		return envVars{}, err
	}
	influxBucket, err := lookupVar("INFLUX_BUCKET")
	if err != nil {
		return envVars{}, err
	}
	return envVars{
		myPort:        myPort,
		influxAddress: influxAddress,
		influxToken:   influxToken,
		influxOrg:     influxOrg,
		influxBucket:  influxBucket,
	}, nil
}

func handleRequest(w *http.ResponseWriter, r *http.Request, influxAPI *api.WriteAPI) {
	// Create a new HTTP request with the same method, URL, and body as the original request
	slaID := r.Header.Get("Sla-Id")
	proxyTs := time.Now()

	r.Header.Add("Proxy-Timestamp", strconv.FormatInt(proxyTs.UnixMilli(), 10))

	// Send the proxy request using the custom transport
	resp, err := http.DefaultTransport.RoundTrip(r)
	if err != nil {
		http.Error(*w, "Error sending proxy request", http.StatusInternalServerError)
		fmt.Println("Error sending proxy request", err)
		return
	}
	respTs := time.Now()
	defer resp.Body.Close()

	// Copy the headers from the proxy response to the original response
	for name, values := range resp.Header {
		for _, value := range values {
			(*w).Header().Add(name, value)
		}
	}

	// Set the status code of the original response to the status code of the proxy response
	(*w).WriteHeader(resp.StatusCode)

	// Copy the body of the proxy response to the original response
	io.Copy((*w), resp.Body)

	p := influxdb2.NewPoint("proxy_send",
		map[string]string{"sla_id": slaID},
		map[string]interface{}{"value": proxyTs},
		respTs)
	(*influxAPI).WritePoint(p)
}

func main() {
	vars, err := initEnvVars()
	if err != nil {
		log.Fatal("Error starting proxy server: ", err)
	}

	client := influxdb2.NewClientWithOptions("http://"+vars.influxAddress, vars.influxToken,
		influxdb2.DefaultOptions().SetBatchSize(20))
	writeAPI := client.WriteAPI(vars.influxOrg, vars.influxBucket)

	server := http.Server{
		Addr: ":" + vars.myPort,
		Handler: http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			handleRequest(&w, r, &writeAPI)
		}),
	}

	log.Println("Starting proxy server on", vars.myPort)
	err = server.ListenAndServe()
	if err != nil {
		log.Fatal("Error starting proxy server: ", err)
	}

	// Force all unwritten data to be sent
	writeAPI.Flush()
	// Ensures background processes finishes
	client.Close()
}
