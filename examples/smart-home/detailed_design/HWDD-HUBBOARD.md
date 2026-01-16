---
id: "HWDD-HUBBOARD"
type: hardware_detailed_design
name: "Hub Main Board Design"
description: >
  Detailed design for central hub main circuit board
satisfies:
  - "HWREQ-HUB"
---

# Hub Main Board Design

## Board Specifications

- Form factor: 100mm x 100mm
- Layers: 4-layer PCB
- Stack-up: Signal-Ground-Power-Signal

## Block Diagram

```mermaid
flowchart TB
    subgraph board["Main Board 100x100mm"]
        subgraph compute["Compute"]
            soc[ARM Cortex-A53<br/>Quad-core]
            ddr[(DDR4 1GB)]
            emmc[(eMMC 8GB)]
        end

        subgraph connectivity["Connectivity"]
            zigbee([Zigbee 3.0])
            wifi([Wi-Fi 802.11ac])
            eth([Ethernet GbE])
            ble([Bluetooth 5.0])
        end

        subgraph power["Power"]
            usbc[USB-C 5V/3A]
            pmic[PMIC]
        end
    end

    soc <--> ddr
    soc <--> emmc
    soc <--> zigbee
    soc <--> wifi
    soc <--> eth
    soc <--> ble
    usbc --> pmic
    pmic --> soc
```

## Power Distribution

- Input: 5V/3A USB-C with PD negotiation
- Rails: 5V, 3.3V, 1.8V, 1.1V (DDR4)
- Efficiency: > 85% at full load

```mermaid
flowchart LR
    usbc[USB-C 5V/3A]

    subgraph psu["Power Supply Unit"]
        pd[PD Controller]
        buck1[Buck 3.3V]
        buck2[Buck 1.8V]
        buck3[Buck 1.1V]
    end

    io[I/O Radios]
    core[SoC Core]
    ddr[(DDR4 Memory)]

    usbc --> pd
    pd --> buck1
    pd --> buck2
    pd --> buck3
    buck1 --> io
    buck2 --> core
    buck3 --> ddr
```

## Layer Stack-up

```mermaid
flowchart TB
    subgraph stackup["4-Layer PCB Stack-up"]
        l1[Layer 1: Signal Top]
        l2[(Layer 2: Ground Plane)]
        l3[(Layer 3: Power Plane)]
        l4[Layer 4: Signal Bottom]
    end

    l1 --- l2
    l2 --- l3
    l3 --- l4
```
