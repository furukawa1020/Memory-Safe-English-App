package service

import (
	"strings"

	"memory-safe-english/services/api/internal/domain"
	"memory-safe-english/services/api/internal/repository"
)

type UserService struct {
	users repository.UserRepository
}

func NewUserService(users repository.UserRepository) UserService {
	return UserService{users: users}
}

func (s UserService) GetMe(userID string) (domain.User, error) {
	if strings.TrimSpace(userID) == "" {
		return domain.User{}, domain.ErrUnauthorized
	}
	return s.users.GetUser(strings.TrimSpace(userID))
}
