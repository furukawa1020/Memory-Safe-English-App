package security

import (
	"net/http/httptest"
	"testing"
	"time"
)

func TestSlidingWindowLimiterBlocksUntilWindowExpires(t *testing.T) {
	limiter := NewSlidingWindowLimiter(2, 10*time.Second)
	now := time.Date(2026, 4, 19, 12, 0, 0, 0, time.UTC)

	if decision := limiter.allowAt("client:ip-1", now); !decision.Allowed {
		t.Fatalf("first request should be allowed")
	}
	if decision := limiter.allowAt("client:ip-1", now.Add(2*time.Second)); !decision.Allowed {
		t.Fatalf("second request should be allowed")
	}

	blocked := limiter.allowAt("client:ip-1", now.Add(4*time.Second))
	if blocked.Allowed {
		t.Fatalf("third request should be blocked")
	}
	if blocked.RetryAfter <= 0 {
		t.Fatalf("expected positive retry after")
	}

	allowedAgain := limiter.allowAt("client:ip-1", now.Add(11*time.Second))
	if !allowedAgain.Allowed {
		t.Fatalf("request after window expiry should be allowed")
	}
}

func TestClientIPFromRequestPrefersForwardedHeaders(t *testing.T) {
	request := httptest.NewRequest("POST", "/auth/login", nil)
	request.RemoteAddr = "10.0.0.10:1234"
	request.Header.Set("X-Forwarded-For", "203.0.113.7, 10.0.0.10")

	if got := ClientIPFromRequest(request); got != "203.0.113.7" {
		t.Fatalf("ClientIPFromRequest() = %q, want %q", got, "203.0.113.7")
	}
}

func TestAttemptLimiterBlocksBySubjectAcrossIPs(t *testing.T) {
	limiter := NewAttemptLimiter(1, time.Minute)

	first := limiter.Allow("198.51.100.10", NormalizeRateLimitSubject("User@example.com"))
	if !first.Allowed {
		t.Fatalf("first attempt should be allowed")
	}

	second := limiter.Allow("203.0.113.10", NormalizeRateLimitSubject("user@example.com"))
	if second.Allowed {
		t.Fatalf("second attempt for same subject should be blocked")
	}
}
