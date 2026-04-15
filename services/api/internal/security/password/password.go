package password

import (
	"crypto/pbkdf2"
	"crypto/rand"
	"crypto/sha256"
	"crypto/subtle"
	"encoding/hex"
	"errors"
	"fmt"
	"strconv"
	"strings"
)

const algorithm = "pbkdf2_sha256"

var ErrInvalidHash = errors.New("invalid password hash")

type Hasher struct {
	Iterations int
	SaltLength int
	KeyLength  int
}

func NewHasher(iterations int) Hasher {
	return Hasher{
		Iterations: iterations,
		SaltLength: 16,
		KeyLength:  32,
	}
}

func (h Hasher) Hash(password string) (string, error) {
	salt := make([]byte, h.SaltLength)
	if _, err := rand.Read(salt); err != nil {
		return "", err
	}

	derived, err := pbkdf2.Key(sha256.New, password, salt, h.Iterations, h.KeyLength)
	if err != nil {
		return "", err
	}

	return fmt.Sprintf("%s$%d$%s$%s", algorithm, h.Iterations, hex.EncodeToString(salt), hex.EncodeToString(derived)), nil
}

func (h Hasher) Verify(password, encoded string) (bool, error) {
	parts := strings.Split(encoded, "$")
	if len(parts) != 4 || parts[0] != algorithm {
		return false, ErrInvalidHash
	}

	iterations, err := strconv.Atoi(parts[1])
	if err != nil {
		return false, ErrInvalidHash
	}
	salt, err := hex.DecodeString(parts[2])
	if err != nil {
		return false, ErrInvalidHash
	}
	expected, err := hex.DecodeString(parts[3])
	if err != nil {
		return false, ErrInvalidHash
	}

	actual, err := pbkdf2.Key(sha256.New, password, salt, iterations, len(expected))
	if err != nil {
		return false, err
	}

	return subtle.ConstantTimeCompare(actual, expected) == 1, nil
}
