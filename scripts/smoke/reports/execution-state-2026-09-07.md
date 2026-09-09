# Execution-state fixes — 2026-09-07

Follow-up: [CoW platform fixes and qualification](cow-platform-fixes-2026-09-09.md) covers Windows private memory, failed-restore lifecycle protection, and the later ARM64 pending-interrupt regression. The backend revision and observations below remain historical.

The Windows ARM64 post-restore command hang is fixed in the tested cases. Linux ARM64 now has working full execution-state capture/restore with VGICv3. Windows x86-64 has a new implementation with successful cross-compilation and executable userspace tests, but no native x86 WHP live qualification. This report does not mark all of #8 complete.

Backend revision: libkrun `51c1ed3b83dc826c02800fb297995538e4eac55d`. Firmware remains `6cca413ac248f63e65d4ea4748b3bc36cd1b22f3`, using matching ARM64 kernel and agentd builds. Microsandbox additionally captures device state before interrupt-controller state, and captures RAM afterward. Public CLI/SDK signatures and disk-only snapshot formats are unchanged. Old Windows ARM64 development full snapshots require recapture because the internal execution-state ABI now includes CPU activity and clock-frequency state.

## What changed

- Windows ARM64: capture and restore the architecture-specific internal activity register, including StartupSuspend. Previously, an online secondary CPU could remain startup-suspended after restore, leaving guest work waiting indefinitely. Preserve genuinely offline CPUs too; do not force all CPUs online.
- Windows ARM64 timers: freeze partition time at the all-vCPU pause barrier. After WHP activates partition time, reinstall saved timer compare/control values before releasing any vCPU. Installing them earlier allows WHP's time activation to rebase the deadline incorrectly. Preserve enable/mask state and validate counter frequency.
- Linux ARM64: enumerate complete variable-width KVM registers, retain MP and exception state, save VGICv3 private/global interrupt state, and retain virtual/physical counter origins and timer controls across pause. Restore VM-wide counter offsets once, not separately for each vCPU. Use the architectural host frequency rather than a nonexistent CNTFRQ GET_ONE_REG interface. Accommodate VGIC initialization ordering on Linux 6.12 without dropping the IIDR handshake.
- Windows x86-64: add register/MSR, XSAVE, local APIC, SynIC, userspace IOAPIC, partition capability/frequency and AP startup state. An already-running restored AP skips SIPI initialization so its saved instruction pointer is not overwritten.
- Shared controller: discard stale capture/restore replies when waiting for a later pause/resume acknowledgement. A failed capture on one CPU must not poison another CPU's source-resume response queue.
- Microsandbox: quiesce/capture device workers before capturing interrupt-controller state. Otherwise the saved queues and saved interrupt state can disagree about a late device completion. KVM's LPI pending-table flush must still precede RAM capture.

## Live coverage

All live VMs below used Alpine, two vCPUs, 256 MiB memory and a 512 MiB root. These are debug builds. Linux ARM64 ran inside Debian 13/Linux 6.12.107 on QEMU/HVF with EL2 exposed on the M5 Max Mac: `/dev/kvm` and nVHE initialization were verified, and Microsandbox used KVM, not software CPU emulation. This qualifies the exercised nested-KVM configuration, not every ARM host or kernel.

| Check | Windows ARM64/WHP | Linux ARM64/KVM |
| --- | --- | --- |
| Running full capture and usable restore | Pass, standard memory | Pass, standard and CoW memory |
| Flat / layered root coverage | Flat clock/offline-CPU tests; layered full lifecycle; earlier flat full lifecycle | Flat standard full lifecycle; layered CoW full lifecycle |
| Idempotent pause/resume, status and paused exec rejection | Pass | Pass |
| Two captures while remaining user-paused | Pass | Pass |
| 15-second pause, boot identity and original workload progress | Pass | Pass |
| Two children, private RAM writes, source/child isolation | Pass | Pass |
| Direct full archive capture/restore and archive unlink survival | Pass | Pass |
| Stop paused source, child still usable | Pass | Pass |
| Six restores from the later paused snapshot; CPU0 and CPU1 pinned timer work on every child | Pass, 6/6 | Pass, 6/6 |
| CPU1 offline at capture, still offline after restore, then successfully onlined | Pass | Pass |
| Delayed restore: first-thaw wall time, elapsed clocks, timer expiration/cancellation | Pass | Pass, CoW |

The clock fixture has four concurrently scheduled readers. Windows observed a 20,638.76 ms wall-time gap with only 46.29 ms monotonic/boottime advancement across the checkpoint boundary; its five-second elapsed-time timer fired once at 5,004.89 ms. Linux observed a 10,519.05 ms wall gap, 96.74 ms elapsed-clock advancement, and one timer expiration at 5,012.80 ms. Both reported no backward readings, expired the absolute wall timer once, canceled the cancel-on-clock-set timer, and passed the first-thaw wall-time bound. These are guest application observations, not just successful host API calls.

macOS/HVF ARM64 and OVH Linux/KVM x86-64 also passed a fresh flat-root CoW lifecycle regression, including 15-second pause, full and repeated paused captures, independent children, direct archive restore and cleanup. This is a regression check of the shared ordering change, not a repeat of every earlier SDK/platform test.

## Observed elapsed times

Milliseconds, individual CLI wall-time observations. The hosts and memory modes differ; these are not controlled speedup comparisons, percentiles, or stop-the-world durations. The first diagnostic nested-KVM restore took 13,947 ms; subsequent lifecycle restores below were much shorter. Retain that cold/diagnostic outlier rather than claiming a stable distribution.

| Operation | Windows ARM64 standard, layered | Linux ARM64 standard, flat | Linux ARM64 CoW, layered |
| --- | ---: | ---: | ---: |
| Running full capture | 7,542.96 | 1,084.90 | 1,238.35 |
| Resident pause | 73.82 | 26.57 | 34.11 |
| First / second paused capture | 2,988.71 / 2,754.49 | 360.17 / 340.20 | 506.82 / 315.73 |
| Resident resume | 81.48 | 71.15 | 87.20 |
| Restore child A / B | 2,647.74 / 3,083.40 | 781.67 / 798.37 | 337.50 / 359.42 |
| Direct full archive capture | 13,993.57 | 2,484.29 | 2,351.41 |
| Direct full archive restore | 4,019.68 | 1,727.62 | 1,090.74 |

Six later-paused-snapshot restores took 2,906–3,296 ms on Windows ARM64 and 786.70–972.59 ms on Linux ARM64. Every child executed the marker read and both CPU-pinned probes successfully. The macOS regression's two CoW children restored in 975.91–1,007.92 ms; the OVH x86 Linux regression's children restored in 165.04–166.40 ms.

## Reproduction and evidence

Use `scripts/smoke/cli/cow-memory-lifecycle.py` as described in the preceding lifecycle report. The runner now waits for the detached application's marker before checking it, rather than assuming runtime readiness means the application's first write has happened.

Build `checkpoint-clock-probe.rs` and `checkpoint-cpu-probe.rs` as static Linux musl binaries for the guest architecture. The portable host runners are `checkpoint-clock.py` and `checkpoint-cpu-state.py`. Their module docstrings specify the required environment variables. The CPU test deliberately offlines CPU1 before capture, checks the offline value after restore, and brings CPU1 online again before running the affinity/timer probe. The clock test runs `checkpoint-clock-analyze.py` against the captured application's observations. Every runner attempts source/child cleanup in `finally`.

Local raw evidence: `/private/tmp/msb-execution-state-20260907/` (ARM64 Linux and Windows), `/private/tmp/mac-arch-fix-cow/` (macOS). OVH evidence: `/home/ubuntu/msb-stack8.ElfKzf/arch-fix-results/`. The nested Linux host retained `/root/stack8/` until shutdown; its test disk remains in `/private/tmp/msb-linux-arm64.lXCCOl/`. Snapshots/cache files remain isolated development artifacts; no test workload is intentionally left running.

## Validation limits

The native ARM64 VMM suite reported 67 passing tests, two ignored tests and two failures in existing MMIO test setup: `test_register_virtio_device` and `test_register_too_many_devices` call `create_irq_chip()` on ARM and fail with EINVAL. The new ARM execution-state tests passed (3), and focused VGIC tests passed (2). The ARM-only vCPU creation test's declaration-order error was fixed. Microsandbox's focused checkpoint suite passed 27 tests. macOS controller barrier tests passed, including the new stale-capture recovery case.

Windows x86 compiled for `x86_64-pc-windows-msvc`. Executable tests on the Surface's x86 emulation passed for restored-running AP startup, pending SIPI, IOAPIC programming/serialization, and stale-capture response recovery. That does not exercise an x86 virtual processor, XSAVE/APIC restoration, or x86 timer delivery under WHP; a native x86 Windows machine is still required.

Windows CoW memory remains explicitly unavailable at the protected shared-cache integration boundary; these fixes qualify standard-memory execution restore, not a Windows CoW cache implementation. Linux ARM64 execution snapshots require VGICv3: VGICv2 does not expose the same complete input-line/latch state interface, and its ordinary boot path remains unchanged. Host suspend, cross-host CPU compatibility, every interrupt injection race, all failure-injection cases and repeated release-build performance distributions remain outside this report.
