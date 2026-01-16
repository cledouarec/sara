---
id: "SWDD-ALERTPUSH"
type: software_detailed_design
name: "Push Notification Service"
description: >
  Design for delivering security alerts via push notifications
satisfies:
  - "SWREQ-PUSHSDK"
---

# Push Notification Service

## Overview

Security alerts are delivered to user mobile devices using Firebase Cloud
Messaging (FCM) for Android and Apple Push Notification Service (APNs) for iOS.

## Architecture

```mermaid
flowchart LR
    Sensor[Sensor Event]
    Hub[Central Hub]
    Gateway[Cloud Gateway]

    subgraph Push["Push Services"]
        FCM[Firebase FCM]
        APNs[Apple APNs]
    end

    Android[Android App]
    iOS[iOS App]

    Sensor --> Hub
    Hub --> Gateway
    Gateway --> FCM
    Gateway --> APNs
    FCM --> Android
    APNs --> iOS
```

## Priority Levels

| Priority | Use Case              | FCM Priority | APNs Priority |
|----------|-----------------------|--------------|---------------|
| Critical | Intrusion, fire       | High         | 10            |
| High     | Door/window open      | High         | 5             |
| Normal   | Battery low           | Normal       | 1             |

## Payload Structure

```json
{
  "notification": {
    "title": "Security Alert",
    "body": "Front door opened"
  },
  "data": {
    "event_type": "door_sensor",
    "device_id": "sensor-front-door",
    "timestamp": "2024-01-15T10:30:00Z",
    "zone": "entry"
  }
}
```

## Retry Policy

- Maximum 3 delivery attempts
- Exponential backoff: 1s, 2s, 4s
- Fall back to SMS for critical alerts if push fails

## Alert Delivery Sequence

```mermaid
sequenceDiagram
    participant Sensor as Door Sensor
    participant Hub as Central Hub
    participant Cloud as Cloud Gateway
    participant Push as FCM/APNs
    participant App as Mobile App

    Sensor->>Hub: Door opened event
    Hub->>Hub: Evaluate security rules
    Hub->>Cloud: Send alert (TLS)
    Cloud->>Push: Route to push service

    alt Delivery successful
        Push->>App: Push notification
        App->>Cloud: Delivery receipt
    else Delivery failed
        Push-->>Cloud: Delivery failed
        Cloud->>Cloud: Retry with backoff
        Cloud->>Push: Retry notification
    end
```
