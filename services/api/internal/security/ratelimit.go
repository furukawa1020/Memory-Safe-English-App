package security

import (
	"crypto/sha256"
	"encoding/hex"
	"net"
	"net/http"
	"strings"
	"sync"
	"time"
)

type RateLimitDecision struct {
	Allowed    bool
	RetryAfter time.Duration
}

type SlidingWindowLimiter struct {
	maxAttempts int
	window      time.Duration

	mu      sync.Mutex
	buckets map[string][]time.Time
}

func NewSlidingWindowLimiter(maxAttempts int, window time.Duration) *SlidingWindowLimiter {
	return &SlidingWindowLimiter{
		maxAttempts: maxAttempts,
		window:      window,
		buckets:     make(map[string][]time.Time),
	}
}

func (l *SlidingWindowLimiter) Allow(key string) RateLimitDecision {
	return l.allowAt(key, time.Now().UTC())
}

func (l *SlidingWindowLimiter) allowAt(key string, now time.Time) RateLimitDecision {
	if l == nil || key == "" {
		return RateLimitDecision{Allowed: true}
	}

	windowStart := now.Add(-l.window)

	l.mu.Lock()
	defer l.mu.Unlock()

	events := l.buckets[key]
	trimmed := events[:0]
	for _, eventTime := range events {
		if !eventTime.Before(windowStart) {
			trimmed = append(trimmed, eventTime)
		}
	}
	events = trimmed

	if len(events) >= l.maxAttempts {
		retryAfter := events[0].Add(l.window).Sub(now)
		if retryAfter < time.Second {
			retryAfter = time.Second
		}
		l.buckets[key] = events
		return RateLimitDecision{
			Allowed:    false,
			RetryAfter: retryAfter,
		}
	}

	events = append(events, now)
	l.buckets[key] = events
	return RateLimitDecision{Allowed: true}
}

type AttemptLimiter struct {
	byClient  *SlidingWindowLimiter
	bySubject *SlidingWindowLimiter
}

func NewAttemptLimiter(maxAttempts int, window time.Duration) *AttemptLimiter {
	return &AttemptLimiter{
		byClient:  NewSlidingWindowLimiter(maxAttempts, window),
		bySubject: NewSlidingWindowLimiter(maxAttempts, window),
	}
}

func NewClientOnlyAttemptLimiter(maxAttempts int, window time.Duration) *AttemptLimiter {
	return &AttemptLimiter{
		byClient: NewSlidingWindowLimiter(maxAttempts, window),
	}
}

func (l *AttemptLimiter) Allow(clientIP, subject string) RateLimitDecision {
	if l == nil {
		return RateLimitDecision{Allowed: true}
	}

	decision := RateLimitDecision{Allowed: true}

	if l.byClient != nil && clientIP != "" {
		current := l.byClient.Allow("client:" + clientIP)
		if !current.Allowed {
			decision = moreRestrictiveDecision(decision, current)
		}
	}

	if l.bySubject != nil && subject != "" {
		current := l.bySubject.Allow("subject:" + HashRateLimitSubject(subject))
		if !current.Allowed {
			decision = moreRestrictiveDecision(decision, current)
		}
	}

	return decision
}

func ClientIPFromRequest(r *http.Request) string {
	if r == nil {
		return ""
	}

	if forwarded := strings.TrimSpace(r.Header.Get("X-Forwarded-For")); forwarded != "" {
		first := strings.Split(forwarded, ",")[0]
		if value := strings.TrimSpace(first); value != "" {
			return value
		}
	}

	if realIP := strings.TrimSpace(r.Header.Get("X-Real-IP")); realIP != "" {
		return realIP
	}

	host, _, err := net.SplitHostPort(strings.TrimSpace(r.RemoteAddr))
	if err == nil {
		return host
	}

	return strings.TrimSpace(r.RemoteAddr)
}

func NormalizeRateLimitSubject(value string) string {
	return strings.ToLower(strings.TrimSpace(value))
}

func HashRateLimitSubject(value string) string {
	sum := sha256.Sum256([]byte(strings.TrimSpace(value)))
	return hex.EncodeToString(sum[:16])
}

func moreRestrictiveDecision(current, next RateLimitDecision) RateLimitDecision {
	if current.Allowed {
		return next
	}
	if next.RetryAfter > current.RetryAfter {
		return next
	}
	return current
}
