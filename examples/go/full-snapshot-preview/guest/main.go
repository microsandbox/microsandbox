// A guest process whose identity, counter, and value exist only in memory.
package main

import (
	"crypto/rand"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"net"
	"os"
	"os/exec"
	"strings"
	"sync"
	"time"
)

const socket = "/dev/shm/go-snapshot.sock"

type State struct {
	ID     string
	PID    int
	BootID string
	Value  string
	Ticks  int64
	Unix   int64
}

func main() {
	if err := run(); err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}
}
func run() error {
	switch os.Args[1] {
	case "daemon":
		cmd := exec.Command(os.Args[0], "serve")
		if err := cmd.Start(); err != nil {
			return err
		}
		fmt.Println(cmd.Process.Pid)
		return nil
	case "serve":
		token := make([]byte, 16)
		if _, err := rand.Read(token); err != nil {
			return err
		}
		boot, err := os.ReadFile("/proc/sys/kernel/random/boot_id")
		if err != nil {
			return err
		}
		state := State{ID: hex.EncodeToString(token), PID: os.Getpid(), BootID: strings.TrimSpace(string(boot)), Value: "initial"}
		var mu sync.Mutex
		go func() {
			for range time.Tick(100 * time.Millisecond) {
				mu.Lock()
				state.Ticks++
				mu.Unlock()
			}
		}()
		l, err := net.Listen("unix", socket)
		if err != nil {
			return err
		}
		defer l.Close()
		for {
			c, err := l.Accept()
			if err != nil {
				return err
			}
			var request string
			err = json.NewDecoder(c).Decode(&request)
			if err == nil {
				mu.Lock()
				if strings.HasPrefix(request, "set:") {
					state.Value = strings.TrimPrefix(request, "set:")
				}
				state.Unix = time.Now().Unix()
				err = json.NewEncoder(c).Encode(state)
				mu.Unlock()
			}
			c.Close()
			if err != nil {
				return err
			}
		}
	default:
		c, err := net.DialTimeout("unix", socket, 3*time.Second)
		if err != nil {
			return err
		}
		defer c.Close()
		c.SetDeadline(time.Now().Add(5 * time.Second))
		if err = json.NewEncoder(c).Encode(os.Args[1]); err != nil {
			return err
		}
		var state State
		if err = json.NewDecoder(c).Decode(&state); err != nil {
			return err
		}
		return json.NewEncoder(os.Stdout).Encode(state)
	}
}
