# <sub><img height="18" src="https://octicons-col.vercel.app/cloud/A770EF">&nbsp;&nbsp;CLOUD HOSTING</sub>

Because of KVM virtualization requirement, `msb` can only work with dedicated servers or cloud instances with nested virtualization enabled. Many cloud providers offer servers that support either or both. Some examples are:

##

#### Bare‑Metal Providers

The best option for self-hosting is a dedicated server.

| #   | Provider                                                                                        | Type                                                                                              | Cheapest Server | Price      | Notes                           |
| --- | ----------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------- | --------------- | ---------- | ------------------------------- |
| 1   | **[Scaleway](https://www.scaleway.com/en/dedibox/start/)**                                      | **[Bare‑Metal](https://www.scaleway.com/en/dedibox/start/)**                                      | START-1-M-SSD   | **≈ $11**  | Unmetered traffic; Paris/Warsaw |
| 2   | **[OVHcloud](https://eco.ovhcloud.com/en/?_gl=1*1fvpjnw*_gcl_au*MTc0MTg2MjQxMy4xNzQ1OTIxOTEx)** | **[Bare‑Metal](https://eco.ovhcloud.com/en/?_gl=1*1fvpjnw*_gcl_au*MTc0MTg2MjQxMy4xNzQ1OTIxOTEx)** | SYS‑LE-1        | **≈ $25**  | 500 Mbps; EU/CA                 |
| 3   | **[Hetzner](https://www.hetzner.com/dedicated-rootserver)**                                     | **[Bare‑Metal](https://www.hetzner.com/dedicated-rootserver)**                                    | AX41            | **≈ $42**  | One‑time setup fee; DE & FI     |
| 4   | **[Vultr](https://www.vultr.com/products/bare-metal/)**                                         | **[Bare‑Metal](https://www.vultr.com/products/bare-metal/)**                                      | c1.small        | **≈ $120** | Hourly billing; global regions  |

##

#### Cloud VPS / VM Providers (Nested Virtualization)

> [!WARNING]
> Nested virtualization (running VMs inside VMs) may have slower performance compared to bare-metal solutions. For production workloads with heavy usage, consider using bare-metal servers for optimal performance.

| #   | Provider                                                                            | Type                                                                         | Cheapest Plan      | Price     | Nested Virt       | Notes                                              |
| --- | ----------------------------------------------------------------------------------- | ---------------------------------------------------------------------------- | ------------------ | --------- | ----------------- | -------------------------------------------------- |
| 1   | **[DigitalOcean](https://www.digitalocean.com/products/droplets)**                  | **[Cloud VPS](https://www.digitalocean.com/products/droplets)**              | Basic Droplet 1 GB | **≈ $4**  | Yes (default)     | Modest perf, all regions                           |
| 2   | **[Google Cloud](https://cloud.google.com/compute)**                                | **[Cloud VM](https://cloud.google.com/compute)**                             | n1‑standard‑1      | **≈ $34** | Yes (enable flag) | Must enable Nested Virtualization; Intel host only |
| 3   | **[Microsoft Azure](https://azure.microsoft.com/en-us/products/virtual-machines/)** | **[Cloud VM](https://azure.microsoft.com/en-us/products/virtual-machines/)** | D2s_v3             | **≈ $70** | Yes (Dv3/Ev3)     | VT‑x exposed by default on Gen‑2 VMs               |
