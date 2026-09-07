//go:build cow_live && microsandbox_ffi_path

package microsandbox

import (
	"context"
	"fmt"
	"os"
	"strings"
	"testing"
	"time"
)

// This exercises the public SDK against a matching development runtime/kernel bundle.
func TestCowResidentCapture(t *testing.T) {
	if os.Getenv("MSB_COW_LIVE") != "1" {
		t.Skip("requires matching live bundle")
	}
	ctx, cancel := context.WithTimeout(context.Background(), 2*time.Minute)
	defer cancel()
	name := fmt.Sprintf("cow8-go-%d", os.Getpid())
	source, err := CreateSandbox(ctx, name, WithImage("alpine"), WithRootDisk(RootDisk.Managed(512)), WithMemory(256), WithMemorySnapshot(MemorySnapshotCow))
	if err != nil {
		t.Fatal(err)
	}
	t.Cleanup(func() {
		if err := source.Stop(context.Background()); err != nil {
			t.Error(err)
		}
	})
	if _, err := source.Exec(ctx, "sh", []string{"-c", "echo source > /dev/shm/sdk-marker"}); err != nil {
		t.Fatal(err)
	}
	if err := source.Pause(ctx); err != nil {
		t.Fatal(err)
	}
	paused, err := GetSandbox(ctx, name)
	if err != nil {
		t.Fatal(err)
	}
	if paused.Status() != SandboxStatusPaused {
		t.Fatalf("got status %s", paused.Status())
	}
	if _, err := Snapshot.Create(ctx, SnapshotCreateOptions{Name: name + "-full", FromSandbox: name, Full: true}); err != nil {
		t.Fatal(err)
	}
	if err := paused.Resume(ctx); err != nil {
		t.Fatal(err)
	}
	child, err := CreateSandbox(ctx, name+"-child", WithFromSnapshot(name+"-full"), WithMemorySnapshot(MemorySnapshotCow))
	if err != nil {
		t.Fatal(err)
	}
	t.Cleanup(func() {
		if err := child.Stop(context.Background()); err != nil {
			t.Error(err)
		}
	})
	result, err := child.Exec(ctx, "cat", []string{"/dev/shm/sdk-marker"})
	if err != nil {
		t.Fatal(err)
	}
	if strings.TrimSpace(result.Stdout()) != "source" {
		t.Fatal("child lost captured memory")
	}
	if _, err := child.Exec(ctx, "sh", []string{"-c", "echo child > /dev/shm/sdk-marker"}); err != nil {
		t.Fatal(err)
	}
	result, err = source.Exec(ctx, "cat", []string{"/dev/shm/sdk-marker"})
	if err != nil {
		t.Fatal(err)
	}
	if strings.TrimSpace(result.Stdout()) != "source" {
		t.Fatal("child changed source memory")
	}
	if err := child.Pause(ctx); err != nil {
		t.Fatal(err)
	}
}
