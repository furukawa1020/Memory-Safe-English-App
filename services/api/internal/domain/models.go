package domain

import "time"

type User struct {
	ID                 string    `json:"user_id"`
	Email              string    `json:"email"`
	DisplayName        string    `json:"display_name"`
	AuthProvider       string    `json:"auth_provider"`
	SubscriptionStatus string    `json:"subscription_status"`
	CreatedAt          time.Time `json:"created_at"`
}

type Session struct {
	ID              string    `json:"session_id"`
	UserID          string    `json:"user_id"`
	Mode            string    `json:"mode"`
	ContentID       string    `json:"content_id,omitempty"`
	StartedAt       time.Time `json:"started_at"`
	CompletedAt     time.Time `json:"completed_at,omitempty"`
	CompletionState string    `json:"completion_state"`
}

type EventLog struct {
	ID         string         `json:"event_id"`
	UserID     string         `json:"user_id"`
	SessionID  string         `json:"session_id"`
	EventType  string         `json:"event_type"`
	Payload    map[string]any `json:"payload"`
	OccurredAt time.Time      `json:"occurred_at"`
	CreatedAt  time.Time      `json:"created_at"`
}

type Content struct {
	ID          string    `json:"content_id"`
	Title       string    `json:"title"`
	ContentType string    `json:"content_type"`
	Level       string    `json:"level"`
	Topic       string    `json:"topic"`
	Language    string    `json:"language"`
	RawText     string    `json:"raw_text"`
	SummaryText string    `json:"summary_text,omitempty"`
	CreatedAt   time.Time `json:"created_at"`
	UpdatedAt   time.Time `json:"updated_at"`
}
