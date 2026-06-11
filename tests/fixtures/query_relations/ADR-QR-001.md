---
id: "ADR-QR-001"
type: architecture_decision_record
name: "Use MQTT for Device Communication"
description: "Decision to use MQTT instead of HTTP polling"
status: accepted
deciders:
  - "Architecture Team"
justifies:
  - "SWDD-QR-001"
---

# Use MQTT for Device Communication

## Context

Devices need reliable, low-latency communication with the hub.

## Decision

Use MQTT as the device-to-hub messaging protocol.
