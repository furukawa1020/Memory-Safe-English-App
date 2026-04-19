package config

import (
	"fmt"
	"os"
	"strconv"
	"time"
)

type Config struct {
	HTTPAddr               string
	AppEnv                 string
	DatabaseURL            string
	AuthTokenSecret        string
	AccessTokenTTL         time.Duration
	RefreshTokenTTL        time.Duration
	PasswordHashIterations int
	AuthRateLimitWindow    time.Duration
	LoginMaxAttempts       int
	RegisterMaxAttempts    int
	RefreshMaxAttempts     int
	WorkerBaseURL          string
	WorkerAPIKey           string
	WorkerSignatureKey     string
	WorkerTimeout          time.Duration
}

func Load() Config {
	return Config{
		HTTPAddr:               getEnv("API_HTTP_ADDR", ":8080"),
		AppEnv:                 getEnv("APP_ENV", "development"),
		DatabaseURL:            getEnv("DATABASE_URL", ""),
		AuthTokenSecret:        getEnv("AUTH_TOKEN_SECRET", "dev-insecure-change-me"),
		AccessTokenTTL:         getEnvDuration("AUTH_ACCESS_TOKEN_TTL", 15*time.Minute),
		RefreshTokenTTL:        getEnvDuration("AUTH_REFRESH_TOKEN_TTL", 7*24*time.Hour),
		PasswordHashIterations: getEnvInt("PASSWORD_HASH_ITERATIONS", 120000),
		AuthRateLimitWindow:    getEnvDuration("AUTH_RATE_LIMIT_WINDOW", 10*time.Minute),
		LoginMaxAttempts:       getEnvInt("AUTH_RATE_LIMIT_LOGIN_MAX_ATTEMPTS", 10),
		RegisterMaxAttempts:    getEnvInt("AUTH_RATE_LIMIT_REGISTER_MAX_ATTEMPTS", 5),
		RefreshMaxAttempts:     getEnvInt("AUTH_RATE_LIMIT_REFRESH_MAX_ATTEMPTS", 20),
		WorkerBaseURL:          getEnv("WORKER_BASE_URL", "http://127.0.0.1:8090"),
		WorkerAPIKey:           getEnv("WORKER_API_KEY", "dev-worker-api-key"),
		WorkerSignatureKey:     getEnv("WORKER_SIGNATURE_KEY", "dev-worker-signature-key"),
		WorkerTimeout:          getEnvDuration("WORKER_TIMEOUT", 5*time.Second),
	}
}

func (c Config) Validate() error {
	if c.HTTPAddr == "" {
		return fmt.Errorf("API_HTTP_ADDR must not be empty")
	}
	if c.AuthTokenSecret == "" {
		return fmt.Errorf("AUTH_TOKEN_SECRET must not be empty")
	}
	if c.AppEnv == "production" && c.AuthTokenSecret == "dev-insecure-change-me" {
		return fmt.Errorf("AUTH_TOKEN_SECRET must be overridden in production")
	}
	if c.AccessTokenTTL <= 0 || c.RefreshTokenTTL <= 0 {
		return fmt.Errorf("token TTLs must be positive")
	}
	if c.PasswordHashIterations < 100000 {
		return fmt.Errorf("PASSWORD_HASH_ITERATIONS must be at least 100000")
	}
	if c.AuthRateLimitWindow <= 0 {
		return fmt.Errorf("AUTH_RATE_LIMIT_WINDOW must be positive")
	}
	if c.LoginMaxAttempts <= 0 {
		return fmt.Errorf("AUTH_RATE_LIMIT_LOGIN_MAX_ATTEMPTS must be positive")
	}
	if c.RegisterMaxAttempts <= 0 {
		return fmt.Errorf("AUTH_RATE_LIMIT_REGISTER_MAX_ATTEMPTS must be positive")
	}
	if c.RefreshMaxAttempts <= 0 {
		return fmt.Errorf("AUTH_RATE_LIMIT_REFRESH_MAX_ATTEMPTS must be positive")
	}
	if c.WorkerBaseURL == "" {
		return fmt.Errorf("WORKER_BASE_URL must not be empty")
	}
	if c.WorkerAPIKey == "" {
		return fmt.Errorf("WORKER_API_KEY must not be empty")
	}
	if c.WorkerSignatureKey == "" {
		return fmt.Errorf("WORKER_SIGNATURE_KEY must not be empty")
	}
	if c.WorkerTimeout <= 0 {
		return fmt.Errorf("WORKER_TIMEOUT must be positive")
	}
	if c.AppEnv == "production" && (c.WorkerAPIKey == "dev-worker-api-key" || c.WorkerSignatureKey == "dev-worker-signature-key") {
		return fmt.Errorf("worker secrets must be overridden in production")
	}
	return nil
}

func getEnv(key, fallback string) string {
	value := os.Getenv(key)
	if value == "" {
		return fallback
	}
	return value
}

func getEnvDuration(key string, fallback time.Duration) time.Duration {
	value := os.Getenv(key)
	if value == "" {
		return fallback
	}

	parsed, err := time.ParseDuration(value)
	if err != nil {
		return fallback
	}
	return parsed
}

func getEnvInt(key string, fallback int) int {
	value := os.Getenv(key)
	if value == "" {
		return fallback
	}

	parsed, err := strconv.Atoi(value)
	if err != nil {
		return fallback
	}
	return parsed
}
