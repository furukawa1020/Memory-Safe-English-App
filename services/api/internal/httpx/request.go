package httpx

import (
	"encoding/json"
	"net/http"
)

func DecodeJSON(r *http.Request, dst any) error {
	defer r.Body.Close()
	decoder := json.NewDecoder(r.Body)
	decoder.DisallowUnknownFields()
	return decoder.Decode(dst)
}

func UserIDFromHeader(r *http.Request) string {
	return r.Header.Get("X-User-ID")
}
