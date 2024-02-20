// Package implements a simple HTTP proxy that adds a `Proxy-Timestamp` in the request header with the timestamp the request is received here.
package main

import (
	"errors"
	"fmt"
	"io"
	"net/http"
	"net/url"
	"os"
	"strconv"
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
	firstRequestID := false
	if requestID == "" {
		firstRequestID = true
		requestID = uuid.NewV4().String()
		r.Header.Add("GIRAFF-Request-ID", requestID)
		env.log.Debug("Creating Giraff request ID", zap.String("requestID", requestID))
	}
	env.log.Debug("Processing request", zap.String("requestID", requestID))

	// If this is the first request, it means it comes from the iot_layer, so needs to be saved as the entrypoint in the fog network, then passed onto the proxy of the appropriate fog node
	var resp *http.Response
	var err error
	if firstRequestID {
		resp, err = transmitRequest(r, r.URL.String(), fmt.Sprintf("http://%s:3128/", r.URL.Hostname()), env)
	} else {
		resp, err = http.DefaultTransport.RoundTrip(r)
	}

	if err != nil {
		http.Error(*ww, fmt.Sprintf("Error sending proxy request; %s", err.Error()), http.StatusInternalServerError)
		env.log.Error("Error sending proxy request", zap.String("requestID", requestID), zap.Error(err))
		return
	}
	defer resp.Body.Close()
	proxyTx := time.Now()

	tags := resp.Header.Get("GIRAFF-Tags")
	resp.Header.Del("GIRAFF-Tags")
	slaID := resp.Header.Get("GIRAFF-Sla-Id")
	resp.Header.Del("GIRAFF-Sla-Id")
	serviceStatus := resp.StatusCode
	if tags == "" {
		tags = "<no-tags>"
	}
	if slaID == "" {
		slaID = "<no-sla-id>"
	}
	if !firstRequestID {
		transmitURL := resp.Header.Get("GIRAFF-Redirect")
		transmitProxyURL := resp.Header.Get("GIRAFF-Redirect-Proxy")

		if resp.StatusCode == 200 {
			if transmitURL != "" {
				for key, values := range r.Header {
					if resp.Header.Get(key) == "" {
						for _, value := range values {
							resp.Header.Add(key, value)
						}
					}
				}
				outReq, err := http.NewRequest(r.Method, r.URL.String(), resp.Body)
				if err != nil {
					env.log.Error("Failed to create the request", zap.String("requestID", requestID), zap.Error(err))
					return
				}
				for key, values := range resp.Header {
					if outReq.Header.Get(key) == "" {
						for _, value := range values {
							outReq.Header.Add(key, value)
						}
					}
				}
				env.log.Debug("Sending transmit...", zap.String("requestID", requestID), zap.String("transmitURL", transmitURL), zap.String("transmitProxyURL", transmitProxyURL))
				// Update the response, previous body will be closed since defered earlier
				resp, err = transmitRequest(outReq, transmitURL, transmitProxyURL, env)
				if err != nil {
					env.log.Error("Failed to transmit the resquest to the next url", zap.String("requestID", requestID), zap.Error(err))
				}
				defer resp.Body.Close()
			} else {
				env.log.Debug("Sent back normally", zap.String("requestID", requestID))
			}
		} else {
			body := "<no-body>"
			b, err := io.ReadAll(resp.Body)
			if err == nil {
				body = string(b)
			}
			env.log.Error("Failed to proxy the request to the next node", zap.String("requestID", requestID), zap.String("transmitURL", transmitURL), zap.String("transmitProxyURL", transmitProxyURL), zap.Int("code", resp.StatusCode), zap.String("body", body))
		}
	}

	for key, values := range resp.Header {
		for _, value := range values {
			(*ww).Header().Add(key, value)
		}
	}
	// (*ww).WriteHeader(resp.StatusCode)
	_, err = io.Copy(*ww, resp.Body)
	if err != nil {
		env.log.Error("Error copying response body", zap.String("requestID", requestID), zap.Error(err))
		http.Error(*ww, fmt.Sprintf("Error copying response body: %v", err), http.StatusInternalServerError)
		return
	}

	p := influxdb2.NewPoint("proxy",
		map[string]string{"sla_id": slaID, "req_id": requestID, "tags": tags, "first_req_id": strconv.FormatBool(firstRequestID), "status": strconv.Itoa(serviceStatus)},
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

	log.Sugar().Debug("Debug enabled")

	client := influxdb2.NewClientWithOptions("http://"+vars.influxAddress, vars.influxToken,
		influxdb2.DefaultOptions().SetBatchSize(20))
	writeAPI := client.WriteAPI(vars.influxOrg, vars.influxBucket)

	server := http.Server{
		Addr: ":" + vars.myPort,
		Handler: http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			handleRequest(&w, r, &vars, &writeAPI)
		}),
	}

	log.Sugar().Info("Starting proxy server on ", vars.myPort)
	err = server.ListenAndServe()
	if err != nil {
		log.Sugar().Fatal("Error starting proxy server: ", err)
	}

	// Force all unwritten data to be sent
	writeAPI.Flush()
	// Ensures background processes finishes
	client.Close()
}
