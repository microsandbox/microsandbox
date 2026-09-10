# Microsandbox snapshots: groups, checkpoints, and the head

This describes the snapshot-group implementation on the development stack, not an already released CLI.

## Start with the snapshot

A snapshot is a saved point you can use to create another sandbox:

```text
Running sandbox
      |
      +-- disk snapshot ----> new VM boots from the saved disk
      |
      +-- full snapshot ----> new VM resumes saved RAM, CPUs, devices, and disk
```

Disk snapshots also work when the source is paused or stopped. Full snapshots require resident execution state: a running or user-paused VM. Each capture produces a new immutable snapshot, even when unchanged disk layers or RAM objects are reused. Exporting a snapshot packages it as a `.msnap` archive; loading an archive installs it without starting a VM.

## A group gives those saved points a local home

A **group** is a namespace containing snapshots and one selected **head**. Member names such as `cp01` are meaningful inside their group. Each snapshot also keeps its portable `snap_...` ID.

```text
~/.microsandbox/snapshots/
|
+-- worker/
|   +-- group.json                 head = snap_B
|   +-- snap_A/
|   |   +-- snapshot.json          ID, parent, disk/state references
|   |   +-- group-member.json      name = cp01
|   |   +-- layers/...             disk-only payload
|   +-- snap_B/
|       +-- snapshot.json          parent = snap_A
|       +-- group-member.json      name = cp02
|       +-- checkpoint/...         full checkpoint payload, when captured full
|
+-- imported/
    +-- group.json
    +-- snap_A/...                 a separate local copy of the same snapshot
```

IDs above are shortened for readability. A disk-only member uses `layers/`; a full member uses `checkpoint/` with its disk layers, RAM objects, and execution/device state. Optional `metadata.json` stores labels.

Groups do not magically make random IDs collision-proof. They keep local addresses separate. Within one group, the same ID with the same descriptor is reusable; the same ID with different descriptor bytes is rejected. A name already used by another member is also rejected. Nothing is silently overwritten. If a global ID resolves to multiple local copies, use the group-qualified address instead.

## Create and restore

```bash
msb create alpine --name worker --memory 512M

# Group defaults to the source sandbox's name: worker.
msb snapshot create cp01 --from worker --full
msb snapshot create cp02 --from worker --full

# A bare group selects its head, currently cp02.
msb create --name latest --from-snapshot worker --forked

# A qualified name selects an exact checkpoint.
msb create --name earlier --from-snapshot worker:cp01 --forked

# You can choose a different group, or let a member name be generated.
msb snapshot create --from worker --group experiments --full
```

`--forked` shares clean restored RAM pages using copy-on-write; child writes remain private. It does not change which snapshot is selected. Omit `--full` at capture for a disk-only snapshot, and omit `--forked` when cold-booting disk state.

## The head moves forward, not sideways by surprise

Snapshots record their actual source ancestry. Neither timestamps, import order, nor an export's `--since` base defines that ancestry.

```text
worker:cp01 ---- worker:cp02 ---- worker:cp03  <- head
                     \
                      +-------- worker:experiment
```

The rules are small:

- Empty group: the first successful publication initializes its head.
- Known descendant of the current head: advance automatically.
- Same member, older member, sibling, unrelated history, or missing ancestry: keep the current head. The capture/import still succeeds.
- Explicit selection: choose any complete installed member, including an older one.

```bash
msb snapshot head worker             # Read the current head ID
msb snapshot head worker:experiment  # Explicitly choose the other branch
msb snapshot head worker:cp01        # Explicitly rewind
```

There is no special `main` branch. The head is a selected snapshot, not a rule for guessing which future branch is preferred.

### What if two siblings arrive together?

```text
                    +---- snapshot A
head: cp02 ---------+
                    +---- snapshot B

A publishes first:  head cp02 -> A
B publishes next:   B is A's sibling, so head stays A

Result: both snapshots exist. Only the first head update wins.
```

Publication checks and head replacement share a per-group lock. The losing sibling is not discarded or reported as a failed capture. If you want B, select it explicitly. Two captures of the *same* source are serialized and record a parent chain; they are not treated as sibling captures.

## Move a history to another machine

```bash
# On the source machine:
msb snapshot save worker:cp01 cp01.msnap
msb snapshot save worker:cp02 cp02.msnap --since worker:cp01

# On the destination machine:
msb snapshot load cp01.msnap --group received
msb snapshot load cp02.msnap --group received --base received:cp01
msb create --name restored --from-snapshot received --forked
```

`--since` omits disk layers and reusable RAM objects supplied by the explicit base. Loading reconstructs a complete owned snapshot; the target does not depend on replaying earlier VMs. Each archive still includes the target's complete memory map and CPU/device state. The `.msnap` archive does not contain a local group's mutable head file: its declared archive head is the import candidate, and the receiving group applies the rules above.

Loading without `--group` creates a fresh generated group. The final stdout line is the installed artifact **path**, not its ID; scripts can pass it as the next `--base`.

Importing an old checkpoint does not rewind an existing group. To deliberately select the imported archive's head:

```bash
msb snapshot load cp01.msnap --group received --set-head
```

Missing historical checkpoints are okay when payload dependencies are complete. But a missing parent may prevent proving a fast-forward. Filling a history hole does not retrospectively select some other retained tip; select that tip explicitly or import it again once its ancestry is known.

Direct archive capture (`snapshot create --archive`) and direct archive restore still skip installed snapshot directories. `msb branch` still creates a local child without publishing a durable snapshot. Neither operation implicitly moves a group's head; a later explicit capture can join a group using the child's recorded ancestry.
