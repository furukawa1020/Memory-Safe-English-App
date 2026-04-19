package app

import (
	"net/http"
	"testing"
	"time"
)

func TestServerLoginRateLimitBlocksRepeatedAttemptsFromSameIP(t *testing.T) {
	cfg := newTestConfig()
	cfg.LoginMaxAttempts = 1
	cfg.AuthRateLimitWindow = time.Minute

	server, err := NewServer(cfg)
	if err != nil {
		t.Fatalf("NewServer() error = %v", err)
	}

	registerTestUser(t, server)

	first := jsonRequestWithHeaders(t, server, http.MethodPost, "/auth/login", "", map[string]any{
		"email":    "user@example.com",
		"password": "wrong-password",
	}, map[string]string{
		"X-Forwarded-For": "198.51.100.10",
	})
	if first.Code != http.StatusUnauthorized {
		t.Fatalf("first login status = %d, body = %s", first.Code, first.Body.String())
	}

	second := jsonRequestWithHeaders(t, server, http.MethodPost, "/auth/login", "", map[string]any{
		"email":    "different@example.com",
		"password": "wrong-password",
	}, map[string]string{
		"X-Forwarded-For": "198.51.100.10",
	})
	if second.Code != http.StatusTooManyRequests {
		t.Fatalf("second login status = %d, body = %s", second.Code, second.Body.String())
	}
	if second.Header().Get("Retry-After") == "" {
		t.Fatalf("expected Retry-After header on rate limited response")
	}
}

func TestServerLoginRateLimitBlocksRepeatedAttemptsForSameSubjectAcrossIPs(t *testing.T) {
	cfg := newTestConfig()
	cfg.LoginMaxAttempts = 1
	cfg.AuthRateLimitWindow = time.Minute

	server, err := NewServer(cfg)
	if err != nil {
		t.Fatalf("NewServer() error = %v", err)
	}

	registerTestUser(t, server)

	first := jsonRequestWithHeaders(t, server, http.MethodPost, "/auth/login", "", map[string]any{
		"email":    "user@example.com",
		"password": "wrong-password",
	}, map[string]string{
		"X-Forwarded-For": "198.51.100.10",
	})
	if first.Code != http.StatusUnauthorized {
		t.Fatalf("first login status = %d, body = %s", first.Code, first.Body.String())
	}

	second := jsonRequestWithHeaders(t, server, http.MethodPost, "/auth/login", "", map[string]any{
		"email":    "USER@example.com",
		"password": "wrong-password",
	}, map[string]string{
		"X-Forwarded-For": "203.0.113.10",
	})
	if second.Code != http.StatusTooManyRequests {
		t.Fatalf("second login status = %d, body = %s", second.Code, second.Body.String())
	}
}
