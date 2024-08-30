// Runs experiments
package main

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"io"
	"log"
	"math"
	"math/rand"
	"net/http"
	"os"
	"strconv"
	"strings"
	"sync"
	"time"

	"github.com/montanaflynn/stats"
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

// A Function on the fog node
type Function struct {
	TargetNode        string
	Mem               int
	CPU               int
	Latency           int
	Duration          int
	DockerFnName      string
	FunctionName      string
	FirstNodeIP       *string
	RequestInterval   int
	Arrival           int
	ReqContent        string
	ColdStartOverhead int
	StopOverhead      int
	InputMaxSize      string
}

// The FunctionProvisioned response of the function being provisioned
type FunctionProvisioned struct {
	FaasIP     string
	FaasPort   int
	FunctionID string
	NodeID     string
}

// The FogNode is part of the struct return when the marketplace is queried about the fog nodes
type FogNode struct {
	IP string
	ID string
}

const putMarketTimeout = 30 * time.Second
const provisionTimeout = 120 * time.Second

var (
	collectorURL              string
	targetNodes               []string
	targetNodeNames           []string
	iotIP                     string
	marketIP                  string
	marketLocalPort           int
	iotLocalPort              int
	nodesIP                   string
	noLatency                 int
	highLatency               int
	lowLatency                int
	functionColdStartOverhead int
	functionStopOverhead      int
	experimentDuration        int
	overrideFunctionIP        string
	overrideFirstNodeIP       string
	dockerRegistry            string
	successes                 int
	errors                    int
	random                    *rand.Rand
	functionDescriptions      []string
	logger                    otelzap.Logger
	_Logger                   zap.Logger
)

// The FunctionPipeline is a description of the functions to deploy on the network
type FunctionPipeline struct {
	Image        string  `json:"image"`
	NextFunction *string `json:"nextFunction,omitempty"`
	Mem          int     `json:"mem,omitempty"`
	CPU          int     `json:"cpu,omitempty"`
	Latency      string  `json:"latency,omitempty"`
	InputMaxSize string  `json:"input_max_size,omitempty"`
}

// The FunctionPipelineDescription is the description of an individual function to deploy on the network
type FunctionPipelineDescription struct {
	Name      string                      `json:"name"`
	Content   string                      `json:"content"`
	NbVarName string                      `json:"nbVarName"`
	First     string                      `json:"first"`
	Pipeline  map[string]FunctionPipeline `json:"pipeline"`
}

var imageRegistry string

func init() {
	var err error
	collectorURL = os.Getenv("COLLECTOR_URL")
	targetNodes = strings.Fields(os.Getenv("TARGET_NODES"))
	targetNodeNames = strings.Fields(os.Getenv("TARGET_NODE_NAMES"))
	iotIP = os.Getenv("IOT_IP")
	marketIP = os.Getenv("MARKET_IP")
	marketLocalPort, err = strconv.Atoi(os.Getenv("MARKET_LOCAL_PORT"))
	iotLocalPort, err = strconv.Atoi(os.Getenv("IOT_LOCAL_PORT"))
	nodesIP = os.Getenv("NODES_IP")
	noLatency, err = strconv.Atoi(os.Getenv("NO_LATENCY"))
	highLatency, err = strconv.Atoi(os.Getenv("HIGH_LATENCY"))
	lowLatency, err = strconv.Atoi(os.Getenv("LOW_LATENCY"))
	functionColdStartOverhead, err = strconv.Atoi(os.Getenv("FUNCTION_COLD_START_OVERHEAD"))
	functionStopOverhead, err = strconv.Atoi(os.Getenv("FUNCTION_STOP_OVERHEAD"))
	experimentDuration, err = strconv.Atoi(os.Getenv("EXPERIMENT_DURATION"))
	overrideFunctionIP = os.Getenv("OVERRIDE_FUNCTION_IP")
	overrideFirstNodeIP = os.Getenv("OVERRIDE_FIRST_NODE_IP")
	dockerRegistry = os.Getenv("DOCKER_REGISTRY")
	functionDescriptions = strings.Fields(os.Getenv("FUNCTION_DESCRIPTIONS"))

	randomSeed := os.Getenv("RANDOM_SEED")
	seed := time.Now().UnixNano()
	if randomSeed != "" {
		var _seed int
		_seed, err = strconv.Atoi(randomSeed)
		seed = int64(_seed)
	}
	random = rand.New(rand.NewSource(seed))
	if err != nil {
		log.Println("There is an error: ", err)
	}

	dev := os.Getenv("DEV")
	var config zap.Config
	if strings.ToLower(dev) == "true" {
		config = zap.NewDevelopmentConfig()
		config.EncoderConfig.EncodeLevel = zapcore.CapitalColorLevelEncoder
	} else {
		config = zap.NewProductionConfig()
	}
	tmp, err := config.Build()
	_Logger = *tmp
	if err != nil {
		log.Println("Failed to setup the zap logger")
	}
}

func initTracer() (func(context.Context) error, error) {
	exp, err := otlptrace.New(
		context.Background(),
		otlptracegrpc.NewClient(
			otlptracegrpc.WithInsecure(),
			otlptracegrpc.WithEndpoint(collectorURL),
		),
	)
	if err != nil {
		return nil, err
	}
	res, err := resource.New(
		context.Background(),
		resource.WithAttributes(
			attribute.String("service.name", "expe"),
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

func generateRand(min, max int) int {
	return rand.Intn(max-min+1) + min
}

func openLoopPoissonProcess(nb int, period int) []float64 {
	nb++
	interArrivalTimes := make([]float64, nb)
	for i := 0; i < nb; i++ {
		interArrivalTimes[i] = random.ExpFloat64()
	}
	sum, _ := stats.Sum(interArrivalTimes)
	scaleFactor := float64(period) / sum
	scaledInterArrivalTimes := make([]float64, nb)
	for i := 0; i < nb; i++ {
		scaledInterArrivalTimes[i] = interArrivalTimes[i] * scaleFactor
	}
	arrivalTimes, _ := stats.CumulativeSum(scaledInterArrivalTimes)
	return arrivalTimes[:len(arrivalTimes)-1]
}

func putRequestFogNode(ctx context.Context, function Function) ([]byte, int, error) {
	ctx, span := otel.Tracer("").Start(
		ctx,
		"put_request_fog_node")
	defer span.End()
	url := fmt.Sprintf("http://%s:%d/api/function", marketIP, marketLocalPort)
	if function.InputMaxSize == "" {
		function.InputMaxSize = "1500 B"
	}
	data := map[string]interface{}{
		"sla": map[string]interface{}{
			"memory":           fmt.Sprintf("%d MB", function.Mem),
			"cpu":              fmt.Sprintf("%d millicpu", function.CPU),
			"latencyMax":       fmt.Sprintf("%d ms", function.Latency),
			"maxReplica":       1,
			"duration":         fmt.Sprintf("%d ms", function.Duration),
			"functionImage":    fmt.Sprintf("%s/%s", dockerRegistry, function.DockerFnName),
			"functionLiveName": function.FunctionName,
			"dataFlow": []map[string]interface{}{
				{
					"from": map[string]string{"dataSource": function.TargetNode},
					"to":   "thisFunction",
				},
			},
			"inputMaxSize": function.InputMaxSize,
		},
		"targetNode": function.TargetNode,
	}

	jsonData, _ := json.Marshal(data)
	req, _ := http.NewRequest("PUT", url, bytes.NewBuffer(jsonData))
	req.Header.Set("Content-Type", "application/json")
	req = req.WithContext(ctx)
	client := &http.Client{Timeout: putMarketTimeout, Transport: otelhttp.NewTransport(http.DefaultTransport)}
	resp, err := client.Do(req)
	if err != nil {
		logger.Ctx(ctx).Error("Failed to put the function")
		return nil, 0, err
	}
	defer resp.Body.Close()
	body, _ := io.ReadAll(resp.Body)
	return body, resp.StatusCode, nil
}

func provisionOneFunction(ctx context.Context, functionID string) ([]byte, int, error) {
	url := fmt.Sprintf("http://%s:%d/api/function/%s", marketIP, marketLocalPort, functionID)
	req, _ := http.NewRequest("POST", url, nil)
	req = req.WithContext(ctx)
	client := &http.Client{Timeout: provisionTimeout, Transport: otelhttp.NewTransport(http.DefaultTransport)}

	resp, err := client.Do(req)
	if err != nil {
		return nil, 0, err
	}
	defer resp.Body.Close()
	body, _ := io.ReadAll(resp.Body)
	return body, resp.StatusCode, nil
}

type ProvChain struct {
	Data   []byte
	Status int
	Err    error
}

func postProvisionChainFunctions(ctx context.Context, urls []FunctionProvisioned) ([][]byte, []int, error) {
	ctx, span := otel.Tracer("").Start(
		ctx,
		"provisionFunctionChain")
	defer span.End()

	var responses [][]byte
	var statusCodes []int
	wg := sync.WaitGroup{}
	ch := make(chan ProvChain, len(urls))
	for _, url := range urls {
		wg.Add(1)
		go func() {
			ctx, span := otel.Tracer("").Start(
				ctx,
				url.FunctionID,
			)
			defer span.End()

			response, statusCode, err := provisionOneFunction(ctx, url.FunctionID)
			ch <- ProvChain{Data: response, Status: statusCode, Err: err}
			wg.Done()
		}()
	}

	wg.Wait()
	close(ch)

	for provFunc := range ch {
		if provFunc.Err != nil {
			return nil, nil, provFunc.Err
		}
		responses = append(responses, provFunc.Data)
		statusCodes = append(statusCodes, provFunc.Status)
	}
	return responses, statusCodes, nil
}

func postRequestChainFunctions(ctx context.Context, urls []FunctionProvisioned) ([][]byte, []int, error) {
	headers := map[string]string{"Content-Type": "application/json"}
	last := len(urls) - 1
	if last == 0 {
		return nil, nil, nil
	}
	var responses [][]byte
	var statusCodes []int
	for ii := 0; ii < last; ii++ {
		url := fmt.Sprintf("http://%s:%d/function/fogfn-%s/reconfigure", urls[ii].FaasIP, urls[ii].FaasPort, urls[ii].FunctionID)
		data := map[string]string{
			"nextFunctionUrl": fmt.Sprintf("http://%s:%d/function/fogfn-%s", urls[ii+1].FaasIP, urls[ii+1].FaasPort, urls[ii+1].FunctionID),
		}
		jsonData, err := json.Marshal(data)
		if err != nil {
			logger.Ctx(ctx).Sugar().Warnf("Something went wrong with the marshalling to on %s: %v", url, err)
			return nil, nil, err
		}
		req, err := http.NewRequest("POST", url, bytes.NewBuffer(jsonData))
		req = req.WithContext(ctx)
		if err != nil {
			logger.Ctx(ctx).Sugar().Warnf("Something went wrong with the request to on %s: %v", url, err)
			return nil, nil, err
		}
		for key, value := range headers {
			req.Header.Set(key, value)
		}
		client := &http.Client{Timeout: provisionTimeout, Transport: otelhttp.NewTransport(http.DefaultTransport)}

		resp, err := client.Do(req)
		if err != nil {
			logger.Ctx(ctx).Sugar().Warnf("Something went wrong contacting openfaas on %s: %v", url, err)
			return nil, nil, err
		}
		defer resp.Body.Close()
		if resp.StatusCode != 200 {
			body, _ := io.ReadAll(resp.Body)
			logger.Ctx(ctx).Sugar().Warnf("Status code is not OK", resp.StatusCode, string(body))
			return nil, nil, err
		}
		body, err := io.ReadAll(resp.Body)
		if err != nil {
			logger.Ctx(ctx).Sugar().Warnf("Something went wrong with the response body on %s: %v", url, err)
			return nil, nil, err
		}
		responses = append(responses, body)
		statusCodes = append(statusCodes, resp.StatusCode)
		time.Sleep(1 * time.Second)
	}
	return responses, statusCodes, nil
}

func putRequestIotEmulation(ctx context.Context, provisioned FunctionProvisioned, function Function) ([]byte, int, error) {
	faasIP := function.FirstNodeIP
	if overrideFirstNodeIP != "" {
		faasIP = &overrideFirstNodeIP
	}
	url := fmt.Sprintf("http://%s:%d/api/cron", iotIP, iotLocalPort)
	headers := map[string]string{"Content-Type": "application/json"}
	data := map[string]interface{}{
		"iotUrl":        fmt.Sprintf("http://%s:%d/api/print", iotIP, iotLocalPort),
		"nodeUrl":       fmt.Sprintf("http://%s:%d/function/fogfn-%s", provisioned.FaasIP, provisioned.FaasPort, provisioned.FunctionID),
		"functionId":    provisioned.FunctionID,
		"tags":          function.FunctionName,
		"initialWaitMs": function.ColdStartOverhead,
		"durationMs":    function.Duration - function.StopOverhead,
		"intervalMs":    function.RequestInterval,
		"firstNodeIp":   faasIP,
		"content":       function.ReqContent,
	}
	jsonData, _ := json.Marshal(data)
	req, _ := http.NewRequest("PUT", url, bytes.NewBuffer(jsonData))
	for key, value := range headers {
		req.Header.Set(key, value)
	}
	client := &http.Client{Timeout: putMarketTimeout}
	req = req.WithContext(ctx)
	resp, err := client.Do(req)
	if err != nil {
		return nil, 0, err
	}
	defer resp.Body.Close()
	body, _ := io.ReadAll(resp.Body)
	return body, resp.StatusCode, nil
}

func registerNewFunctions(functions []Function) (bool, error) {
	time.Sleep(time.Duration(functions[0].Arrival) * time.Second)
	ctx, span := otel.Tracer("").Start(context.Background(), "register_new_function_"+strings.Join(func() []string {
		var functionNames []string
		for _, ff := range functions {
			functionNames = append(functionNames, ff.DockerFnName)
		}
		return functionNames
	}(), ","))
	defer span.End()

	logger.Ctx(ctx).Info("Starting requests", zap.Int("waited_for", functions[0].Arrival))
	var responses []FunctionProvisioned
	var startedAt *time.Time
	for ii := range functions {
		function := functions[ii]
		response, code, err := putRequestFogNode(ctx, function)
		if err != nil {
			logger.Ctx(ctx).Sugar().Warnf("failed to put to the requested fog node")
			return false, err
		}
		if code != 200 {
			logger.Ctx(ctx).Sugar().Warnw("status failed to put to the requested fog node", string(response))
			return false, nil
		}
		if startedAt == nil {
			now := time.Now()
			startedAt = &now
		}
		var responseJSON map[string]interface{}
		json.Unmarshal(response, &responseJSON)
		faasIP := responseJSON["chosen"].(map[string]interface{})["ip"].(string)
		nodeID := responseJSON["chosen"].(map[string]interface{})["bid"].(map[string]interface{})["nodeId"].(string)
		faasPort, err := strconv.Atoi(responseJSON["chosen"].(map[string]interface{})["port"].(string))
		if err != nil {
			logger.Ctx(ctx).Sugar().Warnw("failed to convert the received port to an integer", err)
			return false, nil
		}
		functionID := responseJSON["sla"].(map[string]interface{})["id"].(string)
		responseStruct := FunctionProvisioned{faasIP, faasPort, functionID, nodeID}
		responses = append(responses, responseStruct)
		if ii+1 < len(functions) {
			functions[ii+1].TargetNode = responseStruct.NodeID
			functions[ii+1].FirstNodeIP = &responseStruct.FaasIP
		}
		logger.Ctx(ctx).Sugar().Infof("Reserving... %d/%d", ii+1, len(functions))
	}
	logger.Ctx(ctx).Info(
		"Reserved", zap.String("functions", strings.Join(func() []string {
			var functionNames []string
			for _, ff := range functions {
				functionNames = append(functionNames, ff.FunctionName)
			}
			return functionNames
		}(), ",")))
	duration := functions[0].Duration
	if startedAt == nil || time.Since(*startedAt).Milliseconds()*2 > int64(duration) {
		logger.Ctx(ctx).Sugar().Warnf("Got the reservation, but no time to proceed to use it, stopping there")
		return false, nil
	}
	_, statusCodes, err := postProvisionChainFunctions(ctx, responses)
	if err != nil {
		logger.Ctx(ctx).Sugar().Warnw("Failed to provision:", err)
		return false, err
	}
	for _, httpCode := range statusCodes {
		if httpCode != 200 {
			logger.Ctx(ctx).Sugar().Warnf("Provisioning failed %d", httpCode)
			return false, nil
		}
	}
	logger.Ctx(ctx).Info("Provisioned", zap.String("functions", strings.Join(func() []string {
		var functionNames []string
		for _, ff := range functions {
			functionNames = append(functionNames, ff.FunctionName)
		}
		return functionNames
	}(), ",")))
	_, statusCodes, err = postRequestChainFunctions(ctx, responses)
	if err != nil {
		logger.Ctx(ctx).Sugar().Warnf("Failed to chain functions: none returned")
		return false, err
	}
	for _, httpCode := range statusCodes {
		if httpCode != 200 {
			logger.Ctx(ctx).Sugar().Errorf("Request failed status chained %d", httpCode)
			return false, nil
		}
	}
	logger.Ctx(ctx).Info("Chained", zap.String("functions", strings.Join(func() []string {
		var functionNames []string
		for _, ff := range functions {
			functionNames = append(functionNames, ff.FunctionName)
		}
		return functionNames
	}(), ",")))
	_, codeIot, err := putRequestIotEmulation(ctx, responses[0], functions[0])
	if err != nil {
		return false, err
	}
	if codeIot == 200 {
		logger.Ctx(ctx).Info("Registered cron", zap.String("function_name", functions[0].FunctionName))
		return true, nil
	}
	return false, nil
}

func doRequest(functions []Function) {
	success, err := registerNewFunctions(functions)
	if err != nil {
		logger.Sugar().Error(err)
	}
	if success {
		successes++
	} else {
		errors++
	}
}

func loadFunctionDescriptions() ([]FunctionPipelineDescription, error) {
	var ret []FunctionPipelineDescription
	for _, descFile := range functionDescriptions {
		data, err := os.ReadFile(descFile)
		if err != nil {
			logger.Error("Failed to read the file", zap.String("file", descFile))
			return nil, err
		}
		var desc FunctionPipelineDescription
		err = json.Unmarshal(data, &desc)
		if err != nil {
			logger.Error("Failed to unmarshal the file", zap.String("file", descFile))
			return nil, err
		}
		ret = append(ret, desc)
	}
	return ret, nil
}

func loadFile(filename string) error {
	nodes := make(map[string]string)
	for ii := range targetNodeNames {
		nodes[strings.ReplaceAll(targetNodeNames[ii], "'", "")] = strings.ReplaceAll(targetNodes[ii], "'", "")
	}
	functions, err := loadFunctions(filename)
	if err != nil {
		logger.Sugar().Error("failed to load functions", err)
		return err
	}
	var wg sync.WaitGroup
	response, err := http.Get(fmt.Sprintf("http://%s:%d/api/fog", marketIP, marketLocalPort))
	if err != nil {
		logger.Sugar().Error("Error after query to fog node", err)
		return err
	}
	defer response.Body.Close()
	body, err := io.ReadAll(response.Body)
	if err != nil {
		logger.Sugar().Error("Error after reading the body from the fog node", err)
		return err
	}
	var fognet []json.RawMessage
	fognetwork := map[string]string{}
	err = json.Unmarshal(body, &fognet)
	if err != nil {
		logger.Sugar().Error("Error unmarshalling the body from the fog node", err)
		return err
	}
	for _, val := range fognet {
		var node FogNode
		err = json.Unmarshal(val, &node)
		if err != nil {
			logger.Sugar().Error("Error unmarshalling the rest of the body from the fog node", err)
			return err
		}
		fognetwork[node.ID] = node.IP
	}

	for _, functionSublist := range functions {
		wg.Add(1)
		go func(functionSublist []Function) {
			defer wg.Done()
			for ii := 0; ii < len(functionSublist); ii++ {
				function := &functionSublist[ii]
				function.TargetNode = nodes[strings.ReplaceAll(function.TargetNode, "'", "")]
				if nodesIP != "" {
					function.FirstNodeIP = &nodesIP
				} else {
					tmp := fognetwork[function.TargetNode]
					function.FirstNodeIP = &tmp
				}
			}
			doRequest(functionSublist)
		}(functionSublist)
	}
	wg.Wait()
	return nil
}

func gamma(alpha, beta float64) float64 {
	u := random.Float64()
	x := math.Pow(1-u, 1/alpha)
	y := math.Pow(u, 1/(alpha-1))
	return beta * x * y
}

func saveFile(filename string) error {
	var functions [][]Function
	functionDescriptions, err := loadFunctionDescriptions()
	if err != nil {
		logger.Sugar().Error("failed to load function descriptions")
		return err
	}
	nbFunctions := make([]int, len(functionDescriptions))
	for ii, fnDesc := range functionDescriptions {
		nbFunction, _ := strconv.Atoi(os.Getenv(fnDesc.NbVarName))
		nbFunctions[ii] = nbFunction
	}
	for _, targetNodeName := range targetNodeNames {
		for ii, fnDesc := range functionDescriptions {
			nbFunction := nbFunctions[ii]
			requestIntervals := make([]int, nbFunction)
			for i := 0; i < nbFunction; i++ {
				requestIntervals[i] = int(math.Ceil(math.Abs(100 * gamma(2.35, 15))))
			}
			durations := make([]int, nbFunction)
			for i := 0; i < nbFunction; i++ {
				durations[i] = 60 * 4
			}
			arrivals := make([]int, nbFunction)
			for i := 0; i < nbFunction; i++ {
				arrivals[i] = int(math.Ceil(openLoopPoissonProcess(nbFunction, experimentDuration)[i]))
			}
			for index := 0; index < nbFunction; index++ {
				arrival := arrivals[index]
				duration := durations[index]
				duration = duration + functionColdStartOverhead + functionStopOverhead
				requestInterval := requestIntervals[index]
				fnName := fnDesc.First
				if fnName == "" {
					logger.Warn("no first function specified", zap.String("descName", fnDesc.Name))
				}
				fnChain := make([]Function, 0)
				for {
					fn := fnDesc.Pipeline[fnName]
					latencyStr := os.Getenv(fnDesc.Pipeline[fnName].Latency)
					latency, _ := strconv.Atoi(latencyStr)
					latency = int(math.Ceil(math.Abs(rand.NormFloat64()*float64(latency) + float64(latency)/4)))
					functionName := fnName + "-i" + strconv.Itoa(index) + "-c" + strconv.Itoa(fn.CPU) + "-m" + strconv.Itoa(fn.Mem) + "-l" + strconv.Itoa(latency) + "-a" + strconv.Itoa(arrival) + "-r" + strconv.Itoa(requestInterval) + "-d" + strconv.Itoa(duration)
					fnChain = append(fnChain, Function{
						TargetNode:        targetNodeName,
						Mem:               fn.Mem,
						CPU:               fn.CPU,
						Latency:           latency,
						Duration:          duration,
						DockerFnName:      fn.Image,
						FunctionName:      functionName,
						FirstNodeIP:       nil,
						RequestInterval:   requestInterval,
						Arrival:           arrival,
						ReqContent:        fnDesc.Content,
						ColdStartOverhead: functionColdStartOverhead,
						StopOverhead:      functionStopOverhead,
						InputMaxSize:      fn.InputMaxSize,
					})
					if fn.NextFunction == nil {
						break
					}
					fnName = *fn.NextFunction
				}
				functions = append(functions, fnChain)
			}
		}
	}
	data, err := json.Marshal(functions)
	if err != nil {
		return err
	}
	var functionNames = map[string]int{}
	for _, ff := range functions {
		var functionNamesBis []string
		for _, ii := range ff {
			functionNamesBis = append(functionNamesBis, ii.DockerFnName)
		}
		functionName := fmt.Sprintf("(%s)", strings.Join(functionNamesBis, ","))
		if val, ok := functionNames[functionName]; ok {
			functionNames[functionName] = val + 1
		} else {
			functionNames[functionName] = 1
		}
	}
	logger.Sugar().Info("Saving functions", functionNames)

	err = os.WriteFile(filename, data, 0644)
	_Logger.Sugar().Info("Saved to", filename)
	if err != nil {
		return err
	}
	return nil
}

func loadFunctions(filename string) ([][]Function, error) {
	var functions [][]Function
	data, err := os.ReadFile(filename)
	if err != nil {
		return nil, err
	}
	err = json.Unmarshal(data, &functions)
	if err != nil {
		return nil, err
	}
	var functionNames = map[string]int{}
	for _, ff := range functions {
		var functionNamesBis []string
		for _, ii := range ff {
			functionNamesBis = append(functionNamesBis, ii.DockerFnName)
		}
		functionName := fmt.Sprintf("(%s)", strings.Join(functionNamesBis, ","))
		if val, ok := functionNames[functionName]; ok {
			functionNames[functionName] = val + 1
		} else {
			functionNames[functionName] = 1
		}
	}
	logger.Sugar().Info("Using functions", functionNames)

	return functions, nil
}
func main() {
	if collectorURL != "" {
		shutdown, err := initTracer()
		if err != nil {
			_Logger.Sugar().Fatal("Failed to initialize otel ", err)
		}

		defer shutdown(context.Background())
		_Logger.Info("Initialized otel")
	} else {
		_Logger.Warn("Otel has not been init")
	}
	tmp := otelzap.New(&_Logger, otelzap.WithMinLevel(_Logger.Level()))
	logger = *tmp
	defer logger.Sync()

	envSaveFile := os.Getenv("EXPE_SAVE_FILE")
	envLoadFile := os.Getenv("EXPE_LOAD_FILE")
	logger.Debug("Load/Save", zap.String("saveFile", envSaveFile), zap.String("loadFile", envLoadFile))
	if envSaveFile != "" {
		saveFile(envSaveFile)
	} else if envLoadFile != "" {
		if len(targetNodes) != len(targetNodeNames) {
			logger.Sugar().Fatal("TARGET_NODES and TARGET_NODE_NAMES should have the same length")
		}
		if dockerRegistry == "" {
			dockerRegistry = os.Getenv("IMAGE_REGISTRY")
			logger.Sugar().Warn("DOCKER_REGISTRY is not set, using the IMAGE_REGISTRY")
		}
		logger.Sugar().Info("Using Docker registry: ", dockerRegistry)
		logger.Sugar().Info(fmt.Sprintf("Using market (%s) and iot_emulation(%s)", marketIP, iotIP))
		loadFile(envLoadFile)
		logger.Sugar().Info(fmt.Sprintf("--> Did %d, failed to provision %d functions.", successes, errors))
	} else {
		logger.Sugar().Fatal("Not EXPE_SAVE_FILE nor EXPE_LOAD_FILE were passed, aborting")
		os.Exit(1)
	}
}
