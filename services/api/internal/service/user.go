package service

import (
	"context"
	"strings"

	"memory-safe-english/services/api/internal/domain"
	"memory-safe-english/services/api/internal/repository"
)

type UserService struct {
	users UserReader
}

type UserReader interface {
	repository.UserRepository
}

func NewUserService(users UserReader) UserService {
	return UserService{users: users}
}

func (s UserService) GetMe(ctx context.Context, userID string) (domain.User, error) {
	if strings.TrimSpace(userID) == "" {
		return domain.User{}, domain.ErrUnauthorized
	}
	return s.users.GetUser(ctx, strings.TrimSpace(userID))
}
