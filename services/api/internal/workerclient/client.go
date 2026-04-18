package workerclient

import (
	"bytes"
	"context"
	"crypto/hmac"
	"crypto/sha256"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"strings"
	"time"
)

type Client struct {
	baseURL      string
	apiKey       string
	signatureKey string
	httpClient   *http.Client
	now          func() time.Time
}

type Chunk struct {
	Order        int    `json:"order"`
	Text         string `json:"text"`
	Role         string `json:"role"`
	SkeletonRank int    `json:"skeleton_rank"`
}

type ChunkingResult struct {
	Language string  `json:"language"`
	Chunks   []Chunk `json:"chunks"`
	Summary  string  `json:"summary"`
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

func (c *Client) AnalyzeChunks(ctx context.Context, text, language string) (ChunkingResult, error) {
	requestBody := map[string]string{
		"text":     text,
		"language": language,
	}
	bodyBytes, err := json.Marshal(requestBody)
	if err != nil {
		return ChunkingResult{}, fmt.Errorf("marshal worker request: %w", err)
	}

	timestamp := fmt.Sprintf("%d", c.now().Unix())
	signature := sign(timestamp, bodyBytes, c.signatureKey)

	req, err := http.NewRequestWithContext(ctx, http.MethodPost, c.baseURL+"/analyze/chunks", bytes.NewReader(bodyBytes))
	if err != nil {
		return ChunkingResult{}, fmt.Errorf("build worker request: %w", err)
	}
	req.Header.Set("Content-Type", "application/json")
	req.Header.Set("X-Worker-Api-Key", c.apiKey)
	req.Header.Set("X-Worker-Timestamp", timestamp)
	req.Header.Set("X-Worker-Signature", signature)

	resp, err := c.httpClient.Do(req)
	if err != nil {
		return ChunkingResult{}, fmt.Errorf("call worker: %w", err)
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		body, _ := io.ReadAll(io.LimitReader(resp.Body, 4096))
		return ChunkingResult{}, fmt.Errorf("worker returned status %d: %s", resp.StatusCode, strings.TrimSpace(string(body)))
	}

	var result ChunkingResult
	if err := json.NewDecoder(resp.Body).Decode(&result); err != nil {
		return ChunkingResult{}, fmt.Errorf("decode worker response: %w", err)
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
