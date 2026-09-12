package ffi

// RestoreOptions is deliberately separate from CreateOptions: no image, CPU/RAM
// geometry or startup command can cross the restore entry point.
type RestoreOptions struct {
	Snapshot                    string               `json:"snapshot"`
	CreationProgress            uint64               `json:"creation_progress,omitempty"`
	Forked                      bool                 `json:"forked,omitempty"`
	DiskOnly                    bool                 `json:"disk_only,omitempty"`
	SnapshotBase                string               `json:"snapshot_base,omitempty"`
	User                        string               `json:"user,omitempty"`
	LogLevel                    string               `json:"log_level,omitempty"`
	ExternalMountPolicy         string               `json:"external_mount_policy,omitempty"`
	DangerouslyInheritResources bool                 `json:"dangerously_inherit_resources,omitempty"`
	Volumes                     map[string]MountSpec `json:"volumes,omitempty"`
	CapturedVolumes             []string             `json:"captured_volumes,omitempty"`
	Ports                       []PortBindingOptions `json:"ports,omitempty"`
	Vsock                       []VsockRouteOptions  `json:"vsock,omitempty"`
}
