package app

import (
	"memory-safe-english/services/api/internal/config"
	"memory-safe-english/services/api/internal/handlers"
	"memory-safe-english/services/api/internal/security/password"
	"memory-safe-english/services/api/internal/security/token"
	"memory-safe-english/services/api/internal/service"
	"memory-safe-english/services/api/internal/store/memory"
	"memory-safe-english/services/api/internal/workerclient"
)

type Application struct {
	Config       config.Config
	Store        *memory.Store
	PasswordHash password.Hasher
	TokenManager token.Manager
}

func NewApplication(cfg config.Config) *Application {
	return &Application{
		Config:       cfg,
		Store:        memory.NewStore(),
		PasswordHash: password.NewHasher(cfg.PasswordHashIterations),
		TokenManager: token.NewManager(cfg.AuthTokenSecret, cfg.AccessTokenTTL, cfg.RefreshTokenTTL),
	}
}

func (a *Application) Routes() handlers.RouteSet {
	authService := service.NewAuthService(a.Store, a.Store, a.PasswordHash, a.TokenManager)
	userService := service.NewUserService(a.Store)
	sessionService := service.NewSessionService(a.Store, a.Store)
	workerAnalyzer := workerclient.New(
		a.Config.WorkerBaseURL,
		a.Config.WorkerAPIKey,
		a.Config.WorkerSignatureKey,
		a.Config.WorkerTimeout,
	)
	analysisService := service.NewAnalysisService(workerAnalyzer, workerAnalyzer)
	contentService := service.NewContentService(a.Store, workerAnalyzer, workerAnalyzer)

	return handlers.RouteSet{
		Health:   handlers.NewHealthHandler(),
		Auth:     handlers.NewAuthHandler(authService),
		Me:       handlers.NewMeHandler(userService),
		Session:  handlers.NewSessionHandler(sessionService),
		Analysis: handlers.NewAnalysisHandler(analysisService),
		Content:  handlers.NewContentHandler(contentService),
	}
}
