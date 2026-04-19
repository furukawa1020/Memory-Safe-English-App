package domain

import "time"

type RefreshTokenFamily struct {
	ID        string
	UserID    string
	CreatedAt time.Time
	RevokedAt *time.Time
}

type RefreshSession struct {
	ID                string
	FamilyID          string
	UserID            string
	TokenHash         string
	ExpiresAt         time.Time
	CreatedAt         time.Time
	RevokedAt         *time.Time
	ReplacedByTokenID string
}
