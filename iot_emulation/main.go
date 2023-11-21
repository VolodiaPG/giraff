// Package implements a simple HTTP proxy that adds a `Proxy-Timestamp` in the request header with the timestamp the request is received here.
package main

import (
	"bytes"
	"context"
	"encoding/json"
	"errors"
	"io"
	"math"
	"math/rand"
	"net/http"
	"net/url"
	"os"
	"time"

	influxdb2 "github.com/influxdata/influxdb-client-go/v2"
	"github.com/influxdata/influxdb-client-go/v2/api"
	"github.com/uptrace/opentelemetry-go-extra/otelzap"
	"go.opentelemetry.io/contrib/instrumentation/net/http/otelhttp"
	"go.opentelemetry.io/otel"
	"go.opentelemetry.io/otel/exporters/jaeger"
	"go.opentelemetry.io/otel/propagation"
	"go.opentelemetry.io/otel/sdk/resource"
	sdktrace "go.opentelemetry.io/otel/sdk/trace"
	semconv "go.opentelemetry.io/otel/semconv/v1.4.0"
	"go.uber.org/zap"
	"gopkg.in/validator.v2"
)

type envContext struct {
	MyPort        string
	InfluxAddress string
	InfluxToken   string
	InfluxOrg     string
	InfluxBucket  string
	ProxyPort     string
	CollectorURL  *string
	Dev           bool
	InfluxWriter  api.WriteAPI
	InfluxClient  influxdb2.Client
	Logger        *otelzap.Logger
}

type cronConfig struct {
	FunctionID    string `json:"functionId" validate:"nonnil"`
	IoTURL        string `json:"iotUrl" validate:"nonnil"`
	NodeURL       string `json:"nodeUrl" validate:"nonnil"`
	Tag           string `json:"tag" validate:"nonnil"`
	InitialWaitMs uint   `json:"intialWaitMs"`
	IntervalMs    uint   `json:"intervalMs" validate:"min=1"`
	DurationMs    uint   `json:"durationMs" validate:"min=1"`
	FirstNodeIP   string `json:"firstNodeIp" validate:"nonnil"`
}

type payload struct {
	Tag    string `json:"tag"`
	SentAt uint   `json:"sentAt"`
	From   string `json:"from"`
	To     string `json:"to"`
}

func lookupVar(varName string) (string, error) {
	theVar, unset := os.LookupEnv(varName)
	if !unset {
		return "", errors.New(varName + " variable is not available in the env")
	}
	return theVar, nil
}

func initEnvContext(logger *otelzap.Logger) (envContext, error) {
	myPort, err := lookupVar("PORT")
	if err != nil {
		return envContext{}, err
	}
	influxAddress, err := lookupVar("INFLUX_ADDRESS")
	if err != nil {
		return envContext{}, err
	}
	influxToken, err := lookupVar("INFLUX_TOKEN")
	if err != nil {
		return envContext{}, err
	}
	influxOrg, err := lookupVar("INFLUX_ORG")
	if err != nil {
		return envContext{}, err
	}
	influxBucket, err := lookupVar("INFLUX_BUCKET")
	if err != nil {
		return envContext{}, err
	}
	var collectorURL *string
	collectorURLLocal, err := lookupVar("COLLECTOR_URL")
	if err != nil {
		logger.Sugar().Warn("Missing variable, but will proceed by deactivating the related feature (opentelemetry):", err)
	} else {
		collectorURL = &collectorURLLocal
	}
	dev := false
	devRaw, err := lookupVar("DEV")
	if err != nil {
		logger.Sugar().Warn("Missing variable, but will proceed by deactivating the related feature (dev=false):", err)
	} else {
		dev = devRaw == "TRUE"
	}
	proxyPort, err := lookupVar("PROXY_PORT")
	if err != nil {
		return envContext{}, err
	}

	client := influxdb2.NewClientWithOptions("http://"+influxAddress, influxToken,
		influxdb2.DefaultOptions().SetBatchSize(20))
	writeAPI := client.WriteAPI(influxOrg, influxBucket)

	return envContext{
		MyPort:        myPort,
		InfluxAddress: influxAddress,
		InfluxToken:   influxToken,
		InfluxOrg:     influxOrg,
		InfluxBucket:  influxBucket,
		ProxyPort:     proxyPort,
		CollectorURL:  collectorURL,
		InfluxWriter:  writeAPI,
		InfluxClient:  client,
		Dev:           dev,
		Logger:        logger,
	}, nil
}

func poissonInterval(lambda time.Duration) time.Duration {
	return time.Duration(float64(time.Second) * (-math.Log(1-rand.Float64()) / float64(lambda.Seconds())))
}

func poissonProcess(lambda time.Duration, eventChan chan struct{}, duration time.Duration) {
	startedAt := time.Now()
	for {
		eventChan <- struct{}{}
		time.Sleep(poissonInterval(lambda))
		if time.Since(startedAt) > duration {
			break
		}
	}
	close(eventChan)
}

func ping(env *envContext, config *cronConfig, client *http.Client, ctx *context.Context) error {
	payload := payload{
		Tag:    config.Tag,
		SentAt: uint(time.Now().UnixMicro()),
		From:   "iot_emumation",
		To:     config.NodeURL,
	}
	jsonData, err := json.Marshal(payload)
	if err != nil {
		env.Logger.Error("Failed to serialize the payload for the cron ping:", zap.Error(err))
		return err
	}
	req, err := http.NewRequest("POST", config.NodeURL, bytes.NewBuffer(jsonData))
	if err != nil {
		env.Logger.Error("HTTP POST creation failed:", zap.Error(err))
		return err
	}
	req = req.WithContext(*ctx)
	req.Header.Set("Content-Type", "application/json")
	req.Header.Set("Sla-Id", config.FunctionID)
	resp, err := client.Do(req)
	if err != nil {
		env.Logger.Error("HTTP POST failed:", zap.Error(err))
		return err
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		b, err := io.ReadAll(resp.Body)
		// b, err := ioutil.ReadAll(resp.Body)  Go.1.15 and earlier
		if err != nil {
			env.Logger.Error("Error reading response body", zap.Error(err))
			return err
		}
		env.Logger.Error("Errored response body", zap.String("fn_id", config.FunctionID), zap.String("resp_body", string(b)))
	}

	return nil
}

func handleCron(w *http.ResponseWriter, r *http.Request, env *envContext) {
	if r.Method != "PUT" {
		http.Error(*w, "wrong method, use PUT", http.StatusForbidden)
		return
	}
	decoder := json.NewDecoder(r.Body)
	var config cronConfig
	if err := decoder.Decode(&config); err != nil {
		http.Error(*w, "Cannot unmarshall request json: "+err.Error(), http.StatusBadRequest)
		return
	}
	if err := validator.Validate(config); err != nil {
		http.Error(*w, "Request body validation failed: "+err.Error(), http.StatusBadRequest)
		return
	}

	env.Logger.Info("Registered cron", zap.String("fn_id", config.FunctionID))

	go func() {
		proxyURL, err := url.Parse("http://" + config.FirstNodeIP + ":" + env.ProxyPort)
		if err != nil {
			env.Logger.Fatal("Failed to configure proxy:", zap.Error(err))
			http.Error(*w, "Failed to configure proxy: "+err.Error(), http.StatusInternalServerError)
			return
		}
		httpClient := &http.Client{Transport: otelhttp.NewTransport(&http.Transport{Proxy: http.ProxyURL(proxyURL)})}

		time.Sleep(time.Duration(config.InitialWaitMs) * time.Millisecond)

		eventChan := make(chan struct{})
		go poissonProcess(time.Duration(config.IntervalMs)*time.Millisecond, eventChan, time.Duration(config.DurationMs)*time.Millisecond)

		go func() {
			ctx, span := otel.GetTracerProvider().Tracer("toto").Start(context.Background(), "ping_"+config.FunctionID)
			defer span.End()
			for range eventChan {
				err := ping(env, &config, httpClient, &ctx)
				if err != nil {
					env.Logger.Warn("Ping failed", zap.Error(err))
					p := influxdb2.NewPoint("proxy_send",
						map[string]string{"sla_id": config.FunctionID},
						map[string]interface{}{"value": 1},
						time.Now())
					env.InfluxWriter.WritePoint(p)
				}
			}
			env.Logger.Info("Unregistered cron", zap.String("fn_id", config.FunctionID))
		}()
	}()
}

func initTracer(env *envContext) (func(context.Context) error, error) {
	exp, err := jaeger.New(jaeger.WithCollectorEndpoint(jaeger.WithEndpoint("http://" + *env.CollectorURL + "/api/traces")))
	if err != nil {
		return nil, err
	}
	tp := sdktrace.NewTracerProvider(
		sdktrace.WithBatcher(exp),
		sdktrace.WithResource(resource.NewWithAttributes(
			semconv.SchemaURL,
			semconv.ServiceNameKey.String("iot_emulation"),
			semconv.DeploymentEnvironmentKey.String("production"),
		)),
	)
	otel.SetTracerProvider(tp)
	otel.SetTextMapPropagator(propagation.NewCompositeTextMapPropagator(propagation.TraceContext{}, propagation.Baggage{}))
	return tp.Shutdown, nil
}

func main() {
	config := zap.NewProductionConfig()
	config.Encoding = "console"
	loggerRaw, _ := config.Build()

	vars, err := initEnvContext(nil)
	if err != nil {
		loggerRaw.Sugar().Fatalf("Error starting iot_emulation: ", err)
		return
	}
	// Force all unwritten data to be sent
	defer vars.InfluxWriter.Flush()
	// Ensures background processes finishes
	defer vars.InfluxClient.Close()

	if vars.Dev {
		loggerRaw, _ = zap.NewDevelopment()
	}
	logger := otelzap.New(loggerRaw)
	defer logger.Sync()
	logger.Logger.Debug("Logger in dev mode")
	vars.Logger = logger

	if vars.CollectorURL != nil {
		// Initialize the OpenTelemetry SDK.
		shutdown, err := initTracer(&vars)
		if err != nil {
			logger.Sugar().Fatal("Failed to init tracing with jaeger/opentlp", err)
			return
		}
		defer shutdown(context.Background())
	}

	mux := http.NewServeMux()
	mux.HandleFunc("/api/cron", func(w http.ResponseWriter, r *http.Request) {
		handleCron(&w, r, &vars)
	})

	logger.Sugar().Infof("Starting iot_emulation server on %s", vars.MyPort)
	if vars.CollectorURL != nil {
		handler := otelhttp.NewHandler(mux, "/api/cron")
		err = http.ListenAndServe(":"+vars.MyPort, handler)
		if err != nil {
			logger.Sugar().Fatal("Error starting iot_emulation server: ", err)
		}
	} else {
		err = http.ListenAndServe(":"+vars.MyPort, mux)
		if err != nil {
			logger.Sugar().Fatal("Error starting iot_emulation server: ", err)
		}
	}
}
