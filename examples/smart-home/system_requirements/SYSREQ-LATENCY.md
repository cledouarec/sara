---
id: "SYSREQ-LATENCY"
type: system_requirement
name: "Device Command Latency"
description: >
  Maximum allowed latency for device control commands
specification: >
  The system SHALL deliver device commands within 500ms of user action
derives_from:
  - "SCEN-DIMMER"
---

# Device Command Latency

Commands sent to devices must be delivered with minimal delay to ensure
responsive user experience.

## Rationale

Users expect immediate feedback when controlling lights and other devices.
Delays greater than 500ms create perception of system lag and reduce user
confidence.

## Verification

- Measure round-trip time from button press to device state change
- 95th percentile must be under 500ms on local network
