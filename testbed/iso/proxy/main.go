// Package implements a simple HTTP proxy that adds a `Proxy-Timestamp` in the request header with the timestamp the request is received here.
package main

import (
	"errors"
	"fmt"
	"io"
	"net/http"
	"net/url"
	"os"
	"strings"
	"time"

	uuid "github.com/satori/go.uuid"

	influxdb2 "github.com/influxdata/influxdb-client-go/v2"
	"github.com/influxdata/influxdb-client-go/v2/api"
	"go.uber.org/zap"
	"go.uber.org/zap/zapcore"
)

type envVars struct {
	myPort        string
	influxAddress string
	influxToken   string
	influxOrg     string
	influxBucket  string
	log           *zap.Logger
}

func lookupVar(varName string) (string, error) {
	theVar, unset := os.LookupEnv(varName)
	if !unset {
		return "", errors.New(varName + " variable is not available in the env")
	}
	return theVar, nil
}

func initEnvVars(log *zap.Logger) (envVars, error) {
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
		log:           log,
	}, nil
}

func transmitRequest(r *http.Request, transmitURL string, transmitProxyURL string, env *envVars) (*http.Response, error) {
	if transmitURL == "" {
		return nil, errors.New("URL to transmit is empty; not transmitting")
	}
	var err error
	r.URL, err = url.Parse(transmitURL)

	if err != nil {
		return nil, errors.Join(errors.New("URL to transmit cannot be parsed; not transmitting;"), err)
	}
	transport := http.Transport{}
	proxyURL, err := url.Parse(transmitProxyURL)
	if err != nil {
		env.log.Warn("Failed to parse proxy URL", zap.Error(err))
	} else {
		transport.Proxy = http.ProxyURL(proxyURL)
	}
	client := &http.Client{
		Transport: &transport,
	}

	env.log.Debug("Sending transmit request")

	resp, err := client.Transport.RoundTrip(r)
	if err != nil {
		return nil, errors.Join(errors.New("error sending a transmit proxy request"), err)
	}
	return resp, nil
}

func handleRequest(ww *http.ResponseWriter, r *http.Request, env *envVars, influxAPI *api.WriteAPI) {
	// Create a new HTTP request with the same method, URL, and body as the original request
	proxyRx := time.Now()
	requestID := r.Header.Get("GIRAFF-Request-ID")
	if requestID == "" {
		requestID = uuid.NewV4().String()
		r.Header.Add("GIRAFF-Request-ID", requestID)
		env.log.Debug("Creating Giraff request ID", zap.String("requestID", requestID))
	}
	slaID := r.Header.Get("GIRAFF-Sla-Id")
	tags := r.Header.Get("GIRAFF-Tags")
	env.log.Debug("Processing request", zap.String("requestID", requestID), zap.String("slaID", slaID))

	resp, err := http.DefaultTransport.RoundTrip(r)
	if err != nil {
		http.Error(*ww, fmt.Sprintf("Error sending proxy request; %s", err.Error()), http.StatusInternalServerError)
		env.log.Error("Error sending proxy request", zap.Error(err))
		return
	}
	defer resp.Body.Close()
	proxyTx := time.Now()

	transmitURL := resp.Header.Get("GIRAFF-Redirect")
	transmitProxyURL := resp.Header.Get("GIRAFF-Redirect-Proxy")
	if transmitURL != "" && resp.StatusCode == 200 {
		for key, values := range r.Header {
			if resp.Header.Get(key) == "" {
				for _, value := range values {
					resp.Header.Add(key, value)
				}
			}
		}
		outReq, err := http.NewRequest(r.Method, r.URL.String(), resp.Body)
		if err != nil {
			env.log.Error("Failed to create the request", zap.Error(err))
			return
		}
		for key, values := range resp.Header {
			if outReq.Header.Get(key) == "" {
				for _, value := range values {
					outReq.Header.Add(key, value)
				}
			}
		}
		env.log.Debug("Sending transmit...", zap.String("transmitURL", transmitProxyURL), zap.String("transmitProxyURL", transmitProxyURL))
		resp, err = transmitRequest(outReq, transmitURL, transmitProxyURL, env)
		if err != nil {
			env.log.Error("Failed to transmit the resquest to the next url", zap.Error(err))
		}
		defer resp.Body.Close()
	} else {
		env.log.Debug("Sent back normally")
	}
	for key, values := range resp.Header {
		for _, value := range values {
			(*ww).Header().Add(key, value)
		}
	}
	(*ww).WriteHeader(resp.StatusCode)
	_, err = io.Copy(*ww, resp.Body)
	if err != nil {
		http.Error(*ww, fmt.Sprintf("Error copying response body: %v", err), http.StatusInternalServerError)
		env.log.Error("Error copying response body", zap.Error(err))
		return
	}
	p := influxdb2.NewPoint("proxy",
		map[string]string{"sla_id": slaID, "req_id": requestID, "tags": tags},
		map[string]interface{}{"value": proxyRx},
		proxyTx)
	(*influxAPI).WritePoint(p)
}

func main() {
	dev, err := lookupVar("DEV")
	var config zap.Config
	if err == nil && strings.ToLower(dev) == "true" {
		config = zap.NewDevelopmentConfig()
	} else {
		config = zap.NewProductionConfig()
	}
	config.EncoderConfig.EncodeLevel = zapcore.CapitalColorLevelEncoder
	config.EncoderConfig.EncodeTime = zapcore.ISO8601TimeEncoder
	config.Encoding = "console"

	log, err := config.Build()
	if err != nil {
		panic(err)
	}

	vars, err := initEnvVars(log)
	if err != nil {
		log.Sugar().Fatal("Error starting proxy server: ", err)
	}

	client := influxdb2.NewClientWithOptions("http://"+vars.influxAddress, vars.influxToken,
		influxdb2.DefaultOptions().SetBatchSize(20))
	writeAPI := client.WriteAPI(vars.influxOrg, vars.influxBucket)

	server := http.Server{
		Addr: ":" + vars.myPort,
		Handler: http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			handleRequest(&w, r, &vars, &writeAPI)
		}),
	}

	log.Sugar().Info("Starting proxy server on", vars.myPort)
	err = server.ListenAndServe()
	if err != nil {
		log.Sugar().Fatal("Error starting proxy server: ", err)
	}

	// Force all unwritten data to be sent
	writeAPI.Flush()
	// Ensures background processes finishes
	client.Close()
}
