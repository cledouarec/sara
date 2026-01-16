---
id: "UC-SECURITY"
type: use_case
name: "Security Monitoring"
description: >
  Monitor and manage home security devices including cameras and door sensors
refines:
  - "SOL-SMARTHOME"
---

# Use Case: Security Monitoring

## Actor(s)

- **Primary Actor**: Homeowner (monitors and arms/disarms security system)
- **Secondary Actors**: Security Service (receives alerts for professional monitoring)

## Pre-conditions

- Security devices (sensors, cameras) are registered and online
- User has security admin permissions
- System is in a valid state (Armed Home, Armed Away, or Disarmed)

## Main Success Outcome

User maintains real-time awareness of home security status with immediate
alerts (< 5 seconds) for any triggered sensors, and can remotely arm/disarm
the system with confirmation.

## Key Functional Scope

- **Status Dashboard**: View all sensor states, camera feeds, and system mode
- **Arm/Disarm Control**: Set system to Armed Home, Armed Away, or Disarmed
- **Zone Management**: Configure independent security zones (entry, perimeter, interior)
- **Alert Handling**: Receive push notifications, trigger sirens, notify monitoring service
- **Event Logging**: Record all security events with timestamps for review

## Post-conditions

- **Success Condition**: System reflects requested arm state; all events logged
- **Failure Condition**: System remains in safe state; user notified of issue

## Tier Feature Matrix

| Capability | Starter | Home | Premium |
|------------|---------|------|---------|
| Sensor status & motion alerts | ✓ | ✓ | ✓ |
| Arm/disarm system | ✓ | ✓ | ✓ |
| Camera live view | — | ✓ | ✓ |
| Multiple security zones | — | ✓ | ✓ |
| Voice arm/disarm | — | ✓ | ✓ |
| AI threat detection | — | — | ✓ |
| Professional monitoring integration | — | — | ✓ |
| 30-day cloud recording | — | — | ✓ |
