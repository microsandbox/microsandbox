package msb

import (
	"bytes"
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"net/http"
	"time"
)

// rpcClient is an internal interface for keeping the microsandbox interactions decoupled from the kind of transport being used
type rpcClient interface {
	startSandbox(ctx context.Context, cfg *config, image string, memory int, cpus int) error
	stopSandbox(ctx context.Context, cfg *config) error
	runRepl(ctx context.Context, cfg *config, lang progLang, code string) (*executionResult, error)
	runCommand(ctx context.Context, cfg *config, command string, args []string) (*executionResult, error)
	getMetrics(ctx context.Context, cfg *config) (*sandboxMetrics, error)
}

// rpcMethod represents a JSON-RPC method name
type rpcMethod string

// JSON-RPC method constants
const (
	methodSandboxStart      rpcMethod = "sandbox.start"
	methodSandboxStop       rpcMethod = "sandbox.stop"
	methodSandboxReplRun    rpcMethod = "sandbox.repl.run"
	methodSandboxCommandRun rpcMethod = "sandbox.command.run"
	methodSandboxMetricsGet rpcMethod = "sandbox.metrics.get"
)

// endpoint routing path
const endpointRoute = "/api/v1/rpc"

// JSON-RPC request/response types
type jsonRPCRequest struct {
	JSONRPC string `json:"jsonrpc"`
	Method  string `json:"method"`
	Params  any    `json:"params"`
	ID      string `json:"id,omitempty"`
}

type jsonRPCResponse struct {
	JSONRPC string          `json:"jsonrpc"`
	Result  json.RawMessage `json:"result,omitempty"`
	Error   *jsonRPCError   `json:"error,omitempty"`
	ID      string          `json:"id"`
}

type jsonRPCError struct {
	Code    int    `json:"code"`
	Message string `json:"message"`
	Data    any    `json:"data,omitempty"`
}

// Request parameter types
type startParams struct {
	Namespace string      `json:"namespace"`
	Sandbox   string      `json:"sandbox"`
	Config    startConfig `json:"config"`
}

type startConfig struct {
	Image  string `json:"image"`
	Memory int    `json:"memory"`
	CPUs   int    `json:"cpus"`
}

type stopParams struct {
	Namespace string `json:"namespace"`
	Sandbox   string `json:"sandbox"`
}

type replRunParams struct {
	Namespace string `json:"namespace"`
	Sandbox   string `json:"sandbox"`
	Language  string `json:"language"`
	Code      string `json:"code"`
}

type commandRunParams struct {
	Namespace string   `json:"namespace"`
	Sandbox   string   `json:"sandbox"`
	Command   string   `json:"command"`
	Args      []string `json:"args"`
	Timeout   int      `json:"timeout,omitempty"`
}

type metricsGetParams struct {
	Namespace   string `json:"namespace"`
	SandboxName string `json:"sandbox"`
}

// Response types
type executionResult struct {
	output json.RawMessage `json:"-"` // Store raw JSON for flexible parsing
}

type metricsResult struct {
	Sandboxes []sandboxMetrics `json:"sandboxes"`
}

type sandboxMetrics struct {
	Name        string  `json:"name"`
	Namespace   string  `json:"namespace"`
	Running     bool    `json:"running"`
	CPUUsage    float64 `json:"cpu_usage"`
	MemoryUsage int     `json:"memory_usage"`
	DiskUsage   int     `json:"disk_usage"`
}

var _ rpcClient = &jsonRPCHTTPClient{}

type jsonRPCHTTPClient struct {
	*http.Client
}

func newDefaultJsonRPCHTTPClient() rpcClient {
	return newJsonRPCHTTPClient(
		&http.Client{
			Transport: &http.Transport{
				MaxIdleConns:       10,
				IdleConnTimeout:    30 * time.Second,
				DisableCompression: true,
			},
		},
	)
}

func newJsonRPCHTTPClient(c *http.Client) rpcClient {
	return &jsonRPCHTTPClient{c}
}

func (d *jsonRPCHTTPClient) makeJSONRPCRequest(ctx context.Context, serverURL string, method rpcMethod, params any, apiKey string, logger Logger, reqIdPrd ReqIdProducer) (resp jsonRPCResponse, err error) {
	req := &jsonRPCRequest{
		JSONRPC: "2.0",
		Method:  string(method),
		Params:  params,
	}
	if reqIdPrd != nil {
		req.ID = reqIdPrd()
	}

	logger.Debug("Making JSON-RPC request", "method", string(method), "id", req.ID)

	reqBytes, err := json.Marshal(req)
	if err != nil {
		logger.Error("Failed to marshal JSON-RPC request", "method", string(method), "error", err)
		return resp, fmt.Errorf("%w: %w", ErrMarshalReqFailed, err)
	}

	httpReq, err := http.NewRequestWithContext(ctx, http.MethodPost, fmt.Sprintf("%s%s", serverURL, endpointRoute), bytes.NewReader(reqBytes))
	if err != nil {
		logger.Error("Failed to create HTTP request", "method", string(method), "error", err)
		return resp, fmt.Errorf("%w: %w", ErrCreateRequestFailed, err)
	}

	httpReq.Header.Set("Content-Type", "application/json")
	if apiKey != "" {
		httpReq.Header.Set("Authorization", "Bearer "+apiKey)
	}

	httpResp, err := d.Do(httpReq)
	if err != nil {
		logger.Error("Failed to send HTTP request", "method", string(method), "error", err)
		return resp, fmt.Errorf("%w: %w", ErrSendRequestFailed, err)
	}
	defer func() {
		if closeErr := httpResp.Body.Close(); closeErr != nil && err == nil {
			err = fmt.Errorf("%w: %w", ErrResponseBodyCloseFailed, closeErr)
		}
	}()

	if httpResp.StatusCode != http.StatusOK {
		body, _ := io.ReadAll(httpResp.Body)
		logger.Error("HTTP request failed", "method", string(method), "status", httpResp.StatusCode, "body", string(body))
		return resp, fmt.Errorf("%w: status %d: %s", ErrRequestFailed, httpResp.StatusCode, string(body))
	}

	respBytes, err := io.ReadAll(httpResp.Body)
	if err != nil {
		return resp, fmt.Errorf("%w: %w", ErrReadResponseFailed, err)
	}

	var jsonResp jsonRPCResponse
	if err := json.Unmarshal(respBytes, &jsonResp); err != nil {
		return resp, fmt.Errorf("%w: %w", ErrUnmarshalRespFailed, err)
	}

	if jsonResp.Error != nil {
		logger.Error("JSON-RPC error", "method", string(method), "error", jsonResp.Error.Message, "code", jsonResp.Error.Code)
		return resp, fmt.Errorf("%w: %s", ErrRPCCall, jsonResp.Error.Message)
	}

	logger.Debug("JSON-RPC request completed successfully", "method", string(method), "id", req.ID)
	return jsonResp, nil
}

func (d *jsonRPCHTTPClient) startSandbox(ctx context.Context, cfg *config, image string, memory int, cpus int) error {
	params := startParams{
		Namespace: cfg.namespace,
		Sandbox:   cfg.name,
		Config: startConfig{
			Image:  image,
			Memory: memory,
			CPUs:   cpus,
		},
	}

	cfg.logger.Info("Starting sandbox", "name", cfg.name, "namespace", cfg.namespace, "image", image, "memory", memory, "cpus", cpus)
	_, err := d.makeJSONRPCRequest(ctx, cfg.serverUrl, methodSandboxStart, params, cfg.apiKey, cfg.logger, cfg.reqIDPrd)
	if err == nil {
		cfg.logger.Info("Sandbox started successfully", "name", cfg.name)
	}
	return err
}

func (d *jsonRPCHTTPClient) stopSandbox(ctx context.Context, cfg *config) error {
	params := stopParams{
		Namespace: cfg.namespace,
		Sandbox:   cfg.name,
	}

	cfg.logger.Info("Stopping sandbox", "name", cfg.name, "namespace", cfg.namespace)
	_, err := d.makeJSONRPCRequest(ctx, cfg.serverUrl, methodSandboxStop, params, cfg.apiKey, cfg.logger, cfg.reqIDPrd)
	if err == nil {
		cfg.logger.Info("Sandbox stopped successfully", "name", cfg.name)
	}
	return err
}

func (d *jsonRPCHTTPClient) runRepl(ctx context.Context, cfg *config, lang progLang, code string) (*executionResult, error) {
	params := replRunParams{
		Namespace: cfg.namespace,
		Sandbox:   cfg.name,
		Language:  lang.String(),
		Code:      code,
	}

	cfg.logger.Debug("Executing code in REPL", "sandbox", cfg.name, "language", lang.String())
	resp, err := d.makeJSONRPCRequest(ctx, cfg.serverUrl, methodSandboxReplRun, params, cfg.apiKey, cfg.logger, cfg.reqIDPrd)
	if err != nil {
		return nil, err
	}

	return &executionResult{output: resp.Result}, nil
}

func (d *jsonRPCHTTPClient) runCommand(ctx context.Context, cfg *config, command string, args []string) (*executionResult, error) {
	params := commandRunParams{
		Namespace: cfg.namespace,
		Sandbox:   cfg.name,
		Command:   command,
		Args:      args,
		Timeout:   int(d.Timeout),
	}

	cfg.logger.Debug("Executing command", "sandbox", cfg.name, "command", command, "args", args)
	resp, err := d.makeJSONRPCRequest(ctx, cfg.serverUrl, methodSandboxCommandRun, params, cfg.apiKey, cfg.logger, cfg.reqIDPrd)
	if err != nil {
		return nil, err
	}

	return &executionResult{output: resp.Result}, nil
}

func (d *jsonRPCHTTPClient) getMetrics(ctx context.Context, cfg *config) (*sandboxMetrics, error) {
	params := metricsGetParams{
		Namespace:   cfg.namespace,
		SandboxName: cfg.name,
	}

	cfg.logger.Debug("Getting sandbox metrics", "sandbox", cfg.name)
	resp, err := d.makeJSONRPCRequest(ctx, cfg.serverUrl, methodSandboxMetricsGet, params, cfg.apiKey, cfg.logger, cfg.reqIDPrd)
	if err != nil {
		return nil, err
	}

	var result metricsResult
	if err := json.Unmarshal(resp.Result, &result); err != nil {
		cfg.logger.Error("Failed to unmarshal metrics result", "error", err)
		return nil, fmt.Errorf("%w: %w", ErrUnmarshalMetricsFailed, err)
	}

	// Return the first sandbox (should be the only one for this specific request)
	if len(result.Sandboxes) == 0 {
		return &sandboxMetrics{}, nil
	}

	return &result.Sandboxes[0], nil
}

// --- Error definitions ---
var (
	ErrMarshalReqFailed        = errors.New("failed to marshal request")
	ErrCreateRequestFailed     = errors.New("failed to create request")
	ErrSendRequestFailed       = errors.New("failed to send request")
	ErrResponseBodyCloseFailed = errors.New("failed to close response body")
	ErrReadResponseFailed      = errors.New("failed to read response")
	ErrUnmarshalRespFailed     = errors.New("failed to unmarshal response")
	ErrUnmarshalMetricsFailed  = errors.New("failed to unmarshal metrics result")
	ErrRequestFailed           = errors.New("request failed")
	ErrRPCCall                 = errors.New("RPC error")
)
