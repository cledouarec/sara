---
id: "HWREQ-ZIGBEE"
type: hardware_requirement
name: "Zigbee Radio Module"
description: >
  Hardware requirement for Zigbee wireless communication in smart devices
specification: >
  Smart devices SHALL include a Zigbee 3.0 compliant radio module with
  minimum +8 dBm TX power and -100 dBm RX sensitivity
derives_from:
  - "SYSARCH-COMM"
---

# Zigbee Radio Module

## Specification

Smart devices requiring mesh networking capability SHALL include a Zigbee 3.0
compliant radio module.

## Requirements

| Parameter          | Specification              |
|--------------------|----------------------------|
| Protocol           | Zigbee 3.0 (IEEE 802.15.4) |
| Frequency          | 2.4 GHz ISM band           |
| TX Power           | +8 dBm minimum             |
| RX Sensitivity     | -100 dBm or better         |
| Range (indoor)     | 30 meters minimum          |
| Antenna            | Integrated PCB or U.FL     |

## Certifications

- FCC Part 15
- CE RED
- IC RSS-247

## Reference Components

- Silicon Labs EFR32MG21
- Texas Instruments CC2652R
