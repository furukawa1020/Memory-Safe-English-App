package audit

import "log"

func LogAuthEvent(requestID, event, clientIP, subject string, ok bool, detail string) {
	log.Printf(
		"audit=true category=auth request_id=%s event=%s ok=%t client_ip=%s subject=%s detail=%q",
		safe(requestID),
		safe(event),
		ok,
		safe(clientIP),
		safe(subject),
		detail,
	)
}

func safe(value string) string {
	if value == "" {
		return "-"
	}
	return value
}
