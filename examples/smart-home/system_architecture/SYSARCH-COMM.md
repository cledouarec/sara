---
id: "SYSARCH-COMM"
type: system_architecture
name: "Communication Architecture"
description: >
  System architecture for device communication and alert delivery
satisfies:
  - "SYSREQ-LATENCY"
  - "SYSREQ-ALERT"
---

# Communication Architecture

## Overview

The communication architecture defines how smart devices communicate with the
central hub and how alerts are delivered to users.

## Components

### Local Communication Layer

- MQTT broker running on central hub
- Zigbee mesh network for device-to-hub communication
- Local processing for sub-500ms command latency

### Cloud Communication Layer

- Secure TLS connection to cloud gateway
- Push notification routing via FCM/APNs
- Fallback SMS gateway for critical alerts

## Data Flow - Command Path

```mermaid
sequenceDiagram
    participant App as Mobile App
    participant Cloud as Cloud Gateway
    participant Hub as Central Hub
    participant MQTT as MQTT Broker
    participant Device as Smart Device

    App->>Cloud: Send command (HTTPS)
    Cloud->>Hub: Forward via secure tunnel
    Hub->>MQTT: Publish to device topic
    MQTT->>Device: Deliver command (Zigbee)
    Device->>MQTT: Acknowledge
    MQTT->>Hub: Confirm delivery
    Hub->>Cloud: Status update
    Cloud->>App: Command confirmed
```

## Data Flow - Alert Path

```mermaid
sequenceDiagram
    participant Device as Smart Device
    participant Hub as Central Hub
    participant Cloud as Cloud Gateway
    participant Push as FCM/APNs
    participant App as Mobile App

    Device->>Hub: Sensor event (Zigbee)
    Hub->>Hub: Evaluate alert rules
    Hub->>Cloud: Send alert (TLS)
    Cloud->>Push: Route to push service
    Push->>App: Push notification
    App->>Cloud: Delivery receipt
```
