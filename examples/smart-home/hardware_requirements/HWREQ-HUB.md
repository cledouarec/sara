---
id: "HWREQ-HUB"
type: hardware_requirement
name: "Central Hub Hardware"
description: >
  Hardware requirements for the smart home central hub device
specification: >
  The central hub SHALL include ARM Cortex-A53 quad-core processor, 1GB RAM,
  Ethernet, Wi-Fi, and Zigbee 3.0 radio
derives_from:
  - "SYSARCH-COMM"
---

# Central Hub Hardware

## Specification

The central hub SHALL meet the following hardware specifications to support
local processing and cloud connectivity.

## Compute Requirements

| Parameter       | Minimum Specification     |
|-----------------|---------------------------|
| Processor       | ARM Cortex-A53 quad-core  |
| Clock Speed     | 1.2 GHz                   |
| RAM             | 1 GB DDR4                 |
| Storage         | 8 GB eMMC                 |
| Operating Temp  | 0°C to 40°C               |

## Connectivity

- Ethernet: 10/100/1000 Mbps RJ45
- Wi-Fi: 802.11ac dual-band
- Zigbee: Integrated Zigbee 3.0 radio
- Z-Wave: Z-Wave 700 series module (optional)
- Bluetooth: BLE 5.0 for device provisioning

## Power

- Input: 5V DC, 3A via USB-C
- Power consumption: < 10W typical
