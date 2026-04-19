package app

import (
	"context"

	"memory-safe-english/services/api/internal/config"
	"memory-safe-english/services/api/internal/handlers"
	"memory-safe-english/services/api/internal/security/password"
	"memory-safe-english/services/api/internal/security/token"
	"memory-safe-english/services/api/internal/store/postgres"
	"memory-safe-english/services/api/internal/service"
	"memory-safe-english/services/api/internal/store/memory"
	"memory-safe-english/services/api/internal/workerclient"
)

type Application struct {
	Config       config.Config
	Users        service.UserReader
	Auth         service.AuthStore
	Sessions     service.SessionStore
	Contents     service.ContentStore
	PasswordHash password.Hasher
	TokenManager token.Manager
	closeFn      func(context.Context) error
}

func NewApplication(cfg config.Config) (*Application, error) {
	app := &Application{
		Config:       cfg,
		PasswordHash: password.NewHasher(cfg.PasswordHashIterations),
		TokenManager: token.NewManager(cfg.AuthTokenSecret, cfg.AccessTokenTTL, cfg.RefreshTokenTTL),
	}

	if cfg.DatabaseURL != "" {
		store, err := postgres.NewStore(cfg.DatabaseURL)
		if err != nil {
			return nil, err
		}
		app.Users = store
		app.Auth = store
		app.Sessions = store
		app.Contents = store
		app.closeFn = store.Close
		return app, nil
	}

	store := memory.NewStore()
	app.Users = store
	app.Auth = store
	app.Sessions = store
	app.Contents = store
	return app, nil
}

func (a *Application) Routes() handlers.RouteSet {
	authService := service.NewAuthService(a.Auth, a.Users, a.PasswordHash, a.TokenManager)
	userService := service.NewUserService(a.Users)
	sessionService := service.NewSessionService(a.Users, a.Sessions)
	workerAnalyzer := workerclient.New(
		a.Config.WorkerBaseURL,
		a.Config.WorkerAPIKey,
		a.Config.WorkerSignatureKey,
		a.Config.WorkerTimeout,
	)
	analysisService := service.NewAnalysisService(workerAnalyzer, workerAnalyzer)
	contentService := service.NewContentService(a.Contents, workerAnalyzer, workerAnalyzer)

	return handlers.RouteSet{
		Health:   handlers.NewHealthHandler(),
		Auth:     handlers.NewAuthHandler(authService),
		Me:       handlers.NewMeHandler(userService),
		Session:  handlers.NewSessionHandler(sessionService),
		Analysis: handlers.NewAnalysisHandler(analysisService),
		Content:  handlers.NewContentHandler(contentService),
	}
}

func (a *Application) Close(ctx context.Context) error {
	if a.closeFn == nil {
		return nil
	}
	return a.closeFn(ctx)
}
