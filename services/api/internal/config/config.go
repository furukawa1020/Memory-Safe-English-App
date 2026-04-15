package config

import "os"

type Config struct {
	HTTPAddr string
	AppEnv   string
}

func Load() Config {
	return Config{
		HTTPAddr: getEnv("API_HTTP_ADDR", ":8080"),
		AppEnv:   getEnv("APP_ENV", "development"),
	}
}

func getEnv(key, fallback string) string {
	value := os.Getenv(key)
	if value == "" {
		return fallback
	}
	return value
}
