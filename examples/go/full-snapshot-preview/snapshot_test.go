package snapshot_test

import (
	"context"
	"encoding/json"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"runtime"
	"strings"
	"testing"
	"time"

	msb "github.com/superradcompany/microsandbox/sdk/go"
)

type state struct {
	ID     string
	PID    int
	BootID string
	Value  string
	Ticks  int64
	Unix   int64
}

var guestBinary string

func TestMain(m *testing.M) {
	dir, err := os.MkdirTemp("/tmp", "go-full-")
	if err != nil {
		panic(err)
	}
	os.Setenv("MSB_HOME", filepath.Join(dir, "home"))
	guestBinary = os.Getenv("MSB_GUEST_BINARY")
	if guestBinary == "" {
		guestBinary = filepath.Join(dir, "guest")
		cmd := exec.Command("go", "build", "-o", guestBinary, "./guest")
		cmd.Env = append(os.Environ(), "GOOS=linux", "GOARCH="+runtime.GOARCH, "CGO_ENABLED=0")
		if out, err := cmd.CombinedOutput(); err != nil {
			panic(fmt.Sprintf("guest build: %v %s", err, out))
		}
	}
	fmt.Println("TEST_HOME=" + dir)
	code := m.Run()
	// Retain isolated artifacts/logs for inspection; test cleanup removes its VMs.
	os.Exit(code)
}
func check(t *testing.T, err error) {
	t.Helper()
	if err != nil {
		t.Fatal(err)
	}
}
func call(t *testing.T, ctx context.Context, s *msb.Sandbox, arg string) state {
	t.Helper()
	out, err := s.Exec(ctx, "/usr/local/bin/go-probe", []string{arg}, msb.WithExecTimeout(10*time.Second))
	check(t, err)
	if !out.Success() {
		t.Fatalf("probe %s: exit=%d stderr=%s", arg, out.ExitCode(), out.Stderr())
	}
	var v state
	check(t, json.Unmarshal(out.StdoutBytes(), &v))
	return v
}
func sameProcess(t *testing.T, got, want state) {
	t.Helper()
	if got.ID != want.ID || got.PID != want.PID || got.BootID != want.BootID {
		t.Fatalf("process was cold-booted: got=%+v want=%+v", got, want)
	}
}
func own(t *testing.T, s *msb.Sandbox, name string) {
	t.Helper()
	t.Cleanup(func() {
		ctx, cancel := context.WithTimeout(context.Background(), 30*time.Second)
		defer cancel()
		// Graceful shutdown requires a running guest; Resume is idempotent.
		if err := s.Resume(ctx); err != nil {
			t.Errorf("resume before stopping %s: %v", name, err)
		}
		if err := s.Stop(ctx); err != nil {
			t.Errorf("stop %s: %v", name, err)
		}
		s.Close()
		if err := msb.RemoveSandbox(ctx, name); err != nil {
			t.Errorf("remove %s: %v", name, err)
		}
	})
}
func restore(t *testing.T, ctx context.Context, name, ref string, forked bool) *msb.Sandbox {
	t.Helper()
	opts := []msb.SandboxOption{msb.WithFromSnapshot(ref)}
	if forked {
		opts = append(opts, msb.WithForked())
	}
	start := time.Now()
	s, err := msb.CreateSandbox(ctx, name, opts...)
	check(t, err)
	own(t, s, name)
	t.Logf("restore %s forked=%v: %s", name, forked, time.Since(start))
	return s
}
func read(t *testing.T, ctx context.Context, s *msb.Sandbox, path, want string) {
	t.Helper()
	v, err := s.FS().ReadString(ctx, path)
	check(t, err)
	if v != want {
		t.Fatalf("%s: got %q want %q", path, v, want)
	}
}
func TestFullSnapshot(t *testing.T) {
	for _, root := range []string{"managed", "flat", "tmpfs"} {
		t.Run(root, func(t *testing.T) {
			ctx, cancel := context.WithTimeout(context.Background(), 8*time.Minute)
			defer cancel()
			name := fmt.Sprintf("go-full-%s-%d", root, time.Now().UnixNano())
			disk := msb.RootDisk.Managed(256)
			if root == "flat" {
				disk = msb.RootDisk.Flat(msb.RootDiskFlatOptions{SizeMiB: 256})
			}
			if root == "tmpfs" {
				disk = msb.RootDisk.Tmpfs(msb.RootDiskTmpfsOptions{SizeMiB: 128})
			}
			source, err := msb.CreateSandbox(ctx, name, msb.WithImage("alpine"), msb.WithCPUs(1), msb.WithMemory(512), msb.WithRootDisk(disk))
			check(t, err)
			own(t, source, name)
			check(t, source.FS().CopyFromHost(ctx, guestBinary, "/usr/local/bin/go-probe"))
			out, err := source.Exec(ctx, "chmod", []string{"755", "/usr/local/bin/go-probe"})
			check(t, err)
			if !out.Success() {
				t.Fatal(out.Stderr())
			}
			out, err = source.Exec(ctx, "/usr/local/bin/go-probe", []string{"daemon"})
			check(t, err)
			if !out.Success() {
				t.Fatal(out.Stderr())
			}
			time.Sleep(300 * time.Millisecond)
			original := call(t, ctx, source, "set:captured-memory")
			check(t, source.FS().WriteString(ctx, "/root/disk-marker", "captured-disk"))
			check(t, source.FS().WriteString(ctx, "/dev/shm/ram-marker", "captured-tmpfs"))
			take := func(member string) *msb.SnapshotArtifact {
				t.Helper()
				start := time.Now()
				snap, err := msb.Snapshot.Create(ctx, msb.SnapshotCreateOptions{Name: member, FromSandbox: name, Full: true, RecordIntegrity: true})
				check(t, err)
				t.Logf("full capture %s: %s", member, time.Since(start))
				if snap.Scope() != msb.SnapshotScopeFull {
					t.Fatalf("scope=%s", snap.Scope())
				}
				report, err := snap.Verify(ctx)
				check(t, err)
				if report.Checkpoint == nil {
					t.Fatal("full checkpoint was not verified")
				}
				return snap
			}
			running := take("running")
			sameProcess(t, call(t, ctx, source, "get"), original)
			check(t, source.Pause(ctx))
			check(t, source.Pause(ctx))
			handle, err := msb.GetSandbox(ctx, name)
			check(t, err)
			if handle.Status() != msb.SandboxStatusPaused {
				t.Fatalf("status=%s", handle.Status())
			}
			pausedCtx, pcancel := context.WithTimeout(ctx, 2*time.Second)
			_, err = source.Exec(pausedCtx, "/usr/local/bin/go-probe", []string{"get"})
			pcancel()
			if err == nil {
				t.Fatal("exec unexpectedly accepted while paused")
			}
			if strings.Contains(err.Error(), "deadline") {
				t.Fatalf("paused exec hung instead of rejecting: %v", err)
			}
			paused := take("paused")
			take("paused-again")
			time.Sleep(2 * time.Second)
			check(t, handle.Resume(ctx))
			check(t, handle.Resume(ctx))
			resumed := call(t, ctx, source, "get")
			sameProcess(t, resumed, original)
			if delta := time.Now().Unix() - resumed.Unix; delta < -3 || delta > 3 {
				t.Fatalf("guest wall clock drift=%ds", delta)
			}
			call(t, ctx, source, "set:source-after-snapshot")
			check(t, source.FS().WriteString(ctx, "/root/disk-marker", "source-after-snapshot"))
			check(t, source.FS().WriteString(ctx, "/dev/shm/ram-marker", "source-after-snapshot"))
			for i, forked := range []bool{false, true} {
				ref := running.Path()
				if forked {
					ref = name + ":paused"
				}
				child := restore(t, ctx, fmt.Sprintf("%s-child%d", name, i), ref, forked)
				v := call(t, ctx, child, "get")
				sameProcess(t, v, original)
				if v.Value != "captured-memory" {
					t.Fatalf("RAM value=%q", v.Value)
				}
				read(t, ctx, child, "/root/disk-marker", "captured-disk")
				read(t, ctx, child, "/dev/shm/ram-marker", "captured-tmpfs")
				call(t, ctx, child, "set:child-private")
				check(t, child.FS().WriteString(ctx, "/root/disk-marker", "child-private"))
				check(t, child.FS().WriteString(ctx, "/dev/shm/ram-marker", "child-private"))
				if call(t, ctx, source, "get").Value != "source-after-snapshot" {
					t.Fatal("child changed source RAM")
				}
				read(t, ctx, source, "/root/disk-marker", "source-after-snapshot")
				check(t, child.Pause(ctx))
				desc, err := child.Branch(ctx, fmt.Sprintf("%s-desc%d", name, i))
				check(t, err)
				own(t, desc, fmt.Sprintf("%s-desc%d", name, i))
				dv := call(t, ctx, desc, "get")
				sameProcess(t, dv, original)
				if dv.Value != "child-private" {
					t.Fatal("branch lost private RAM")
				}
				check(t, child.Resume(ctx))
				// Assert eventual progress, allowing the hosted runner to schedule the guest.
				before := dv.Ticks
				started := time.Now()
				for {
					after := call(t, ctx, desc, "get")
					if after.Ticks > before {
						t.Logf("branch progress forked=%v: ticks %d -> %d after %s", forked, before, after.Ticks, time.Since(started))
						break
					}
					if time.Since(started) >= 5*time.Second {
						t.Fatalf("restored workload did not progress: ticks %d -> %d after %s", before, after.Ticks, time.Since(started))
					}
					time.Sleep(100 * time.Millisecond)
				}
			}
			// Export/import and direct-archive restore must preserve the same live process.
			archive := filepath.Join(t.TempDir(), "full.msb")
			check(t, msb.Snapshot.Save(ctx, paused.Path(), archive, msb.SnapshotSaveOptions{WithParents: true, WithImage: true}))
			loaded, err := msb.Snapshot.LoadWithOptions(ctx, archive, msb.SnapshotLoadOptions{Group: name + "-import"})
			check(t, err)
			imported := restore(t, ctx, name+"-import-child", loaded.Path(), true)
			iv := call(t, ctx, imported, "get")
			sameProcess(t, iv, original)
			if iv.Value != "captured-memory" {
				t.Fatal("export/import lost RAM")
			}
			directPath := filepath.Join(t.TempDir(), "direct.msb")
			direct, err := msb.Snapshot.CreateArchive(ctx, msb.SnapshotArchiveOptions{SnapshotCreateOptions: msb.SnapshotCreateOptions{Name: "direct", FromSandbox: name, Full: true}, ArchivePath: directPath})
			check(t, err)
			archived := restore(t, ctx, name+"-archive-child", direct.Path(), true)
			av := call(t, ctx, archived, "get")
			sameProcess(t, av, original)
			if av.Value != "source-after-snapshot" {
				t.Fatal("direct archive lost current RAM")
			}
			check(t, os.Remove(direct.Path()))
			sameProcess(t, call(t, ctx, archived, "get"), original)
			check(t, source.Pause(ctx)) // Cleanup resumes the paused source before graceful shutdown.
		})
	}
}
