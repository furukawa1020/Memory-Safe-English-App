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
	AuthTokenSecret        string
	AccessTokenTTL         time.Duration
	RefreshTokenTTL        time.Duration
	PasswordHashIterations int
}

func Load() Config {
	return Config{
		HTTPAddr:               getEnv("API_HTTP_ADDR", ":8080"),
		AppEnv:                 getEnv("APP_ENV", "development"),
		AuthTokenSecret:        getEnv("AUTH_TOKEN_SECRET", "dev-insecure-change-me"),
		AccessTokenTTL:         getEnvDuration("AUTH_ACCESS_TOKEN_TTL", 15*time.Minute),
		RefreshTokenTTL:        getEnvDuration("AUTH_REFRESH_TOKEN_TTL", 7*24*time.Hour),
		PasswordHashIterations: getEnvInt("PASSWORD_HASH_ITERATIONS", 120000),
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
