package main

import (
	"log"

	"memory-safe-english/services/api/internal/app"
	"memory-safe-english/services/api/internal/config"
)

func main() {
	cfg := config.Load()

	server, err := app.NewServer(cfg)
	if err != nil {
		log.Fatalf("create server: %v", err)
	}

	log.Printf("api server listening on %s", cfg.HTTPAddr)
	if err := server.ListenAndServe(); err != nil {
		log.Fatalf("listen and serve: %v", err)
	}
}
