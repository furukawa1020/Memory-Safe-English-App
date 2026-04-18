package workerclient

import (
	"bytes"
	"context"
	"crypto/hmac"
	"crypto/sha256"
	"encoding/hex"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"net/http"
	"strings"
	"time"

	"memory-safe-english/services/api/internal/domain"
	"memory-safe-english/services/api/internal/httpx"
)

type Client struct {
	baseURL      string
	apiKey       string
	signatureKey string
	httpClient   doer
	now          func() time.Time
}

type doer interface {
	Do(req *http.Request) (*http.Response, error)
}

func New(baseURL, apiKey, signatureKey string, timeout time.Duration) *Client {
	return &Client{
		baseURL:      strings.TrimRight(baseURL, "/"),
		apiKey:       apiKey,
		signatureKey: signatureKey,
		httpClient:   &http.Client{Timeout: timeout},
		now:          func() time.Time { return time.Now().UTC() },
	}
}

func (c *Client) AnalyzeChunks(ctx context.Context, text, language string) (domain.ChunkingResult, error) {
	requestBody := map[string]string{
		"text":     text,
		"language": language,
	}
	bodyBytes, err := json.Marshal(requestBody)
	if err != nil {
		return domain.ChunkingResult{}, fmt.Errorf("marshal worker request: %w", err)
	}

	timestamp := fmt.Sprintf("%d", c.now().Unix())
	signature := sign(timestamp, bodyBytes, c.signatureKey)

	req, err := http.NewRequestWithContext(ctx, http.MethodPost, c.baseURL+"/analyze/chunks", bytes.NewReader(bodyBytes))
	if err != nil {
		return domain.ChunkingResult{}, fmt.Errorf("build worker request: %w", err)
	}
	req.Header.Set("Content-Type", "application/json")
	req.Header.Set("X-Worker-Api-Key", c.apiKey)
	req.Header.Set("X-Worker-Timestamp", timestamp)
	req.Header.Set("X-Worker-Signature", signature)
	if requestID := httpx.RequestID(ctx); requestID != "" {
		req.Header.Set("X-Request-ID", requestID)
	}

	resp, err := c.httpClient.Do(req)
	if err != nil {
		if errors.Is(err, context.DeadlineExceeded) || errors.Is(err, context.Canceled) {
			return domain.ChunkingResult{}, fmt.Errorf("%w: worker request timed out", domain.ErrUnavailable)
		}
		return domain.ChunkingResult{}, fmt.Errorf("%w: call worker: %v", domain.ErrUnavailable, err)
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		body, _ := io.ReadAll(io.LimitReader(resp.Body, 4096))
		return domain.ChunkingResult{}, UpstreamError{
			StatusCode: resp.StatusCode,
			Message:    strings.TrimSpace(string(body)),
		}
	}

	var result domain.ChunkingResult
	if err := json.NewDecoder(resp.Body).Decode(&result); err != nil {
		return domain.ChunkingResult{}, fmt.Errorf("%w: decode worker response: %v", domain.ErrUnavailable, err)
	}
	if len(result.Chunks) == 0 && result.Summary == "" {
		return domain.ChunkingResult{}, fmt.Errorf("%w: worker returned empty analysis", domain.ErrUnavailable)
	}
	return result, nil
}

func sign(timestamp string, body []byte, key string) string {
	mac := hmac.New(sha256.New, []byte(key))
	_, _ = mac.Write([]byte(timestamp))
	_, _ = mac.Write([]byte("."))
	_, _ = mac.Write(body)
	return hex.EncodeToString(mac.Sum(nil))
}
