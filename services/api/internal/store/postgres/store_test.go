package postgres

import "testing"

func TestNewIDUsesPrefix(t *testing.T) {
	got := newID("usr")
	if len(got) <= len("usr_") || got[:4] != "usr_" {
		t.Fatalf("expected prefix usr_, got %q", got)
	}
}
