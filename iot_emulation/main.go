// Package implements a simple HTTP proxy that adds a `Proxy-Timestamp` in the request header with the timestamp the request is received here.
package main

import (
	"bytes"
	"context"
	"encoding/json"
	"errors"
	"io"
	"io/fs"
	"math"
	"math/rand"
	"mime/multipart"
	"net/http"
	"net/url"
	"os"
	"path/filepath"
	"strings"
	"time"

	"github.com/go-playground/validator/v10"
	influxdb2 "github.com/influxdata/influxdb-client-go/v2"
	"github.com/influxdata/influxdb-client-go/v2/api"
	"github.com/uptrace/opentelemetry-go-extra/otelzap"
	"go.opentelemetry.io/contrib/instrumentation/net/http/otelhttp"
	"go.opentelemetry.io/otel"
	"go.opentelemetry.io/otel/attribute"
	"go.opentelemetry.io/otel/exporters/otlp/otlptrace"
	"go.opentelemetry.io/otel/exporters/otlp/otlptrace/otlptracegrpc"
	"go.opentelemetry.io/otel/propagation"
	"go.opentelemetry.io/otel/sdk/resource"
	sdktrace "go.opentelemetry.io/otel/sdk/trace"
	"go.uber.org/zap"
	"go.uber.org/zap/zapcore"
)

type envContext struct {
	MyPort             string
	InfluxAddress      string
	InfluxToken        string
	InfluxOrg          string
	InfluxBucket       string
	ProxyPort          string
	CollectorURL       *string
	FolderResources    map[string][]fs.DirEntry
	FolderResourceName string
	Dev                bool
	InfluxWriter       api.WriteAPI
	InfluxClient       influxdb2.Client
	Logger             *otelzap.Logger
	Validator          *validator.Validate
}

type cronConfig struct {
	FunctionID    string     `json:"functionId"`
	IoTURL        string     `json:"iotUrl"`
	NodeURL       string     `json:"nodeUrl"`
	Tags          string     `json:"tags"`
	InitialWaitMs uint       `json:"intialWaitMs"`
	IntervalMs    uint       `json:"intervalMs" validate:"min=1"`
	DurationMs    uint       `json:"durationMs" validate:"min=1"`
	FirstNodeIP   string     `json:"firstNodeIp"`
	Content       reqContent `json:"content"`
}

type reqContent struct {
	inner reqContentType
}

type reqContentType interface {
	NewRequest(env *envContext, config *cronConfig) (*http.Request, error)
}

type contentPing struct{}
type contentAudio struct{}
type contentImage struct{}

type payload struct {
	Tag    string `json:"tag"`
	SentAt uint   `json:"sentAt"`
	From   string `json:"from"`
	To     string `json:"to"`
}

func (content contentPing) NewRequest(_ *envContext, config *cronConfig) (*http.Request, error) {
	payload := payload{
		Tag:    config.Tags,
		SentAt: uint(time.Now().UnixMicro()),
		From:   "iot_emumation",
		To:     config.NodeURL,
	}
	jsonData, err := json.Marshal(payload)
	if err != nil {
		return nil, err
	}
	req, err := http.NewRequest("POST", config.NodeURL, bytes.NewBuffer(jsonData))
	if err != nil {
		return nil, err
	}
	req.Header.Set("Content-Type", "application/json")
	return req, nil
}

func (content contentAudio) NewRequest(env *envContext, config *cronConfig) (*http.Request, error) {
	index := uint(rand.Uint32()) % uint(len(env.FolderResources["audios"]))
	data, err := os.Open(filepath.Join(env.FolderResourceName, "audios", env.FolderResources["audios"][index].Name()))
	if err != nil {
		return nil, err
	}
	req, err := http.NewRequest("POST", config.NodeURL, data)
	if err != nil {
		return nil, err
	}
	return req, nil
}

func (content contentImage) NewRequest(env *envContext, config *cronConfig) (*http.Request, error) {
	index := uint(rand.Uint32()) % uint(len(env.FolderResources["images"]))
	filename := env.FolderResources["images"][index].Name()
	data, err := os.Open(filepath.Join(env.FolderResourceName, "images", filename))
	if err != nil {
		return nil, err
	}
	defer data.Close()

	var b bytes.Buffer
	w := multipart.NewWriter(&b)

	part, err := w.CreateFormFile("file", filename)
	if err != nil {
		return nil, err
	}

	if _, err = io.Copy(part, data); err != nil {
		return nil, err
	}

	w.Close()

	req, err := http.NewRequest("POST", config.NodeURL, &b)
	if err != nil {
		return nil, err
	}
	req.Header.Set("Content-Type", w.FormDataContentType())

	return req, nil
}

func (content *reqContent) UnmarshalJSON(b []byte) error {
	var s string
	if err := json.Unmarshal(b, &s); err != nil {
		return err
	}
	switch s {
	case "ping":
		content.inner = contentPing{}
	case "audio":
		content.inner = contentAudio{}
	case "image":
		content.inner = contentImage{}
	}

	return nil
}

func lookupVar(varName string) (string, error) {
	theVar, unset := os.LookupEnv(varName)
	if !unset {
		return "", errors.New(varName + " variable is not available in the env")
	}
	return theVar, nil
}

func initEnvContext(logger *zap.Logger) (envContext, error) {
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
		dev = strings.ToLower(devRaw) == "true"
	}
	proxyPort, err := lookupVar("PROXY_PORT")
	if err != nil {
		return envContext{}, err
	}
	folderResources := make(map[string][]fs.DirEntry)
	folder, err := lookupVar("FOLDER_RESOURCES")
	if err != nil {
		return envContext{}, err
	}
	subdirs, err := os.ReadDir(folder)
	if err != nil {
		logger.Error("Failed to read the content directory")
		return envContext{}, err
	}
	for _, dir := range subdirs {
		if dir.IsDir() {
			files, err := os.ReadDir(filepath.Join(folder, dir.Name()))
			if err != nil {
				logger.Error("Failed to read the content directory")
				return envContext{}, err
			}
			folderResources[dir.Name()] = files
		}
	}

	client := influxdb2.NewClientWithOptions("http://"+influxAddress, influxToken,
		influxdb2.DefaultOptions().SetBatchSize(20))
	writeAPI := client.WriteAPI(influxOrg, influxBucket)

	validate := validator.New()

	return envContext{
		MyPort:             myPort,
		InfluxAddress:      influxAddress,
		InfluxToken:        influxToken,
		InfluxOrg:          influxOrg,
		InfluxBucket:       influxBucket,
		ProxyPort:          proxyPort,
		CollectorURL:       collectorURL,
		InfluxWriter:       writeAPI,
		InfluxClient:       client,
		FolderResources:    folderResources,
		Dev:                dev,
		Logger:             nil,
		Validator:          validate,
		FolderResourceName: folder,
	}, nil
}

// TODO remove arbitrary stuff
func poissonInterval(lambda time.Duration) time.Duration {
	return time.Duration(float64(time.Second) * 10 * (-math.Log(1-rand.Float64()) / float64(lambda.Seconds())))
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
	req, err := config.Content.inner.NewRequest(env, config)
	if err != nil {
		env.Logger.Error("HTTP POST creation failed:", zap.Error(err))
		return err
	}
	req.Header.Set("GIRAFF-Tags", config.Tags)
	req.Header.Set("GIRAFF-Sla-Id", config.FunctionID)
	req = req.WithContext(*ctx)
	resp, err := client.Do(req)
	if err != nil {
		env.Logger.Error("HTTP POST failed:", zap.Error(err))
		return err
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		b, err := io.ReadAll(resp.Body)
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
	if err := env.Validator.Struct(config); err != nil {
		http.Error(*w, "Request body validation failed: "+err.Error(), http.StatusBadRequest)
		return
	}

	ctx, span := otel.Tracer("").Start(r.Context(), "cron_"+config.FunctionID)
	defer span.End()

	env.Logger.Ctx(ctx).Info("Registered cron", zap.String("fn_id", config.FunctionID))

	go func() {
		proxyURL, err := url.Parse("http://" + config.FirstNodeIP + ":" + env.ProxyPort + "/proxy")
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
			for range eventChan {
				ctx, span := otel.Tracer("").Start(context.Background(), "ping_"+config.FunctionID)
				err := ping(env, &config, httpClient, &ctx)
				if err != nil {
					env.Logger.Warn("Ping failed", zap.Error(err))
					p := influxdb2.NewPoint("proxy_send",
						map[string]string{"sla_id": config.FunctionID},
						map[string]interface{}{"value": 1},
						time.Now())
					env.InfluxWriter.WritePoint(p)
				}
				span.End()
			}
			env.Logger.Info("Unregistered cron", zap.String("fn_id", config.FunctionID))
		}()
	}()
}

func initTracer(env *envContext) (func(context.Context) error, error) {
	exp, err := otlptrace.New(
		context.Background(),
		otlptracegrpc.NewClient(
			otlptracegrpc.WithInsecure(),
			otlptracegrpc.WithEndpoint(*env.CollectorURL),
		),
	)
	if err != nil {
		return nil, err
	}
	res, err := resource.New(
		context.Background(),
		resource.WithAttributes(
			attribute.String("service.name", "iot_emulation"),
			attribute.String("library.language", "go"),
		),
	)
	if err != nil {
		return nil, err
	}
	otel.SetTracerProvider(
		sdktrace.NewTracerProvider(
			sdktrace.WithSampler(sdktrace.AlwaysSample()),
			sdktrace.WithBatcher(exp),
			sdktrace.WithResource(res),
		),
	)
	otel.SetTextMapPropagator(propagation.NewCompositeTextMapPropagator(propagation.TraceContext{}, propagation.Baggage{}))
	return exp.Shutdown, nil
}

func main() {
	dev, err := lookupVar("DEV")
	var config zap.Config
	if err == nil && strings.ToLower(dev) == "true" {
		config = zap.NewDevelopmentConfig()
		config.EncoderConfig.EncodeLevel = zapcore.CapitalColorLevelEncoder
	} else {
		config = zap.NewProductionConfig()
	}
	loggerRaw, _ := config.Build()

	vars, err := initEnvContext(loggerRaw)
	if err != nil {
		loggerRaw.Sugar().Fatalf("Error starting iot_emulation: %s", err)
		return
	}
	// Force all unwritten data to be sent
	defer vars.InfluxWriter.Flush()
	// Ensures background processes finishes
	defer vars.InfluxClient.Close()

	if vars.CollectorURL != nil {
		// Initialize the OpenTelemetry SDK.
		shutdown, err := initTracer(&vars)
		if err != nil {
			loggerRaw.Sugar().Fatal("Failed to init tracing with jaeger/opentlp", err)
			return
		}
		defer shutdown(context.Background())
		logger := otelzap.New(loggerRaw, otelzap.WithMinLevel(loggerRaw.Level()))
		defer logger.Sync()
		vars.Logger = logger
		vars.Logger.Info("Initialized otel")
	}

	mux := http.NewServeMux()
	mux.HandleFunc("/api/cron", func(w http.ResponseWriter, r *http.Request) {
		handleCron(&w, r, &vars)
	})

	vars.Logger.Sugar().Infof("Starting iot_emulation server on %s", vars.MyPort)
	if vars.CollectorURL != nil {
		vars.Logger.Info("Starting server w/ otel")
		handler := otelhttp.NewHandler(mux, "/api/cron")
		err = http.ListenAndServe(":"+vars.MyPort, handler)
		if err != nil {
			vars.Logger.Sugar().Fatal("Error starting iot_emulation server: ", err)
		}
	} else {
		vars.Logger.Info("Starting server w/o otel")
		err = http.ListenAndServe(":"+vars.MyPort, mux)
		if err != nil {
			vars.Logger.Sugar().Fatal("Error starting iot_emulation server: ", err)
		}
	}
}
