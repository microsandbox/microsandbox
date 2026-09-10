//go:build cow_live && microsandbox_ffi_path

package microsandbox

import (
	"context"
	"fmt"
	"os"
	"path/filepath"
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
	source, err := CreateSandbox(ctx, name, WithImage("alpine"), WithRootDisk(RootDisk.Managed(512)), WithMemory(256))
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
	branched, err := paused.Branch(ctx, name+"-paused-branch")
	if err != nil {
		t.Fatal(err)
	}
	t.Cleanup(func() {
		if err := branched.Stop(context.Background()); err != nil {
			t.Error(err)
		}
		branched.Close()
	})
	branchResult, err := branched.Exec(ctx, "cat", []string{"/dev/shm/sdk-marker"})
	if err != nil {
		t.Fatal(err)
	}
	if strings.TrimSpace(branchResult.Stdout()) != "source" {
		t.Fatal("branch lost captured RAM")
	}
	snapshot, err := Snapshot.Create(ctx, SnapshotCreateOptions{Name: name + "-full", FromSandbox: name, Full: true})
	if err != nil {
		t.Fatal(err)
	}
	if _, err := os.Stat(filepath.Join(snapshot.Path(), "snapshot.json")); err != nil {
		t.Fatal(err)
	}
	if err := paused.Resume(ctx); err != nil {
		t.Fatal(err)
	}
	// The returned artifact path selects the exact member in its snapshot group.
	child, err := CreateSandbox(ctx, name+"-child", WithFromSnapshot(snapshot.Path()), WithForked())
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
	descendant, err := child.Branch(ctx, name+"-branch")
	if err != nil {
		t.Fatal(err)
	}
	t.Cleanup(func() {
		if err := descendant.Stop(context.Background()); err != nil {
			t.Error(err)
		}
		descendant.Close()
	})
	branchResult, err = descendant.Exec(ctx, "cat", []string{"/dev/shm/sdk-marker"})
	if err != nil {
		t.Fatal(err)
	}
	if strings.TrimSpace(branchResult.Stdout()) != "child" {
		t.Fatal("branch lost private writes")
	}
}
