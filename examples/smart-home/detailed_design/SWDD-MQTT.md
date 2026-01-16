---
id: "SWDD-MQTT"
type: software_detailed_design
name: "MQTT Communication Protocol"
description: >
  Design for device-to-hub communication using MQTT messaging protocol
satisfies:
  - "SWREQ-MQTTCLIENT"
---

# MQTT Communication Protocol

## Overview

All smart devices communicate with the central hub using MQTT (Message Queuing
Telemetry Transport) protocol over the local network.

## Topic Structure

```mermaid
flowchart TB
    subgraph Topics["MQTT Topic Hierarchy"]
        root["smarthome/"]
        device["{device_id}/"]
        status["status"]
        command["command"]
        ack["ack"]
    end

    root --> device
    device --> status
    device --> command
    device --> ack

    note1["Device publishes"] -.-> status
    note2["Hub publishes"] -.-> command
    note3["Device acknowledges"] -.-> ack
```

## QoS Levels

- Status updates: QoS 0 (at most once)
- Commands: QoS 1 (at least once)
- Critical alerts: QoS 2 (exactly once)

## Message Format

```json
{
  "timestamp": "2024-01-15T10:30:00Z",
  "device_id": "light-001",
  "action": "set_brightness",
  "value": 30,
  "correlation_id": "cmd-12345"
}
```

## Connection Management

- Devices maintain persistent connections to the MQTT broker
- Keep-alive interval: 30 seconds
- Automatic reconnection with exponential backoff

## Message Sequence

```mermaid
sequenceDiagram
    participant App as Mobile App
    participant Hub as Central Hub
    participant Broker as MQTT Broker
    participant Device as Smart Device

    App->>Hub: Set brightness command
    Hub->>Broker: Publish to device/command
    Broker->>Device: Deliver command (QoS 1)
    Device->>Broker: PUBACK
    Device->>Device: Apply brightness
    Device->>Broker: Publish to device/ack
    Broker->>Hub: Deliver ack
    Hub->>App: Command confirmed
```
