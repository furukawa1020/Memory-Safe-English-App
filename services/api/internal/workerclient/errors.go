package workerclient

import "fmt"

type UpstreamError struct {
	StatusCode int
	Message    string
}

func (e UpstreamError) Error() string {
	return fmt.Sprintf("worker returned status %d: %s", e.StatusCode, e.Message)
}
