package httpx

import "strings"

func SessionAction(path string) (sessionID, action string, ok bool) {
	trimmed := strings.Trim(path, "/")
	parts := strings.Split(trimmed, "/")
	if len(parts) != 3 || parts[0] != "sessions" {
		return "", "", false
	}
	return parts[1], parts[2], true
}
