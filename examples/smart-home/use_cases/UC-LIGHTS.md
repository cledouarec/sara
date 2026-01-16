---
id: "UC-LIGHTS"
type: use_case
name: "Lighting Control"
description: >
  Control and automate smart lighting throughout the home
refines:
  - "SOL-SMARTHOME"
---

# Use Case: Lighting Control

## Actor(s)

- **Primary Actor**: Homeowner (controls lights for daily use)
- **Secondary Actors**: Guest User (limited control with temporary permissions)

## Pre-conditions

- At least one smart light is registered in the system
- User has appropriate permissions for the target light(s)
- Hub is powered and connected to the network

## Main Success Outcome

User successfully controls light state (on/off, brightness, color) with
visual confirmation in the app and physical change in the light within
2 seconds of command.

## Key Functional Scope

- **Device Control**: Turn lights on/off, adjust brightness 0-100%, set color/temperature
- **Group Management**: Create and control light groups by room or zone
- **Scene Automation**: Define and trigger lighting presets (e.g., "Movie Mode", "Wake Up")
- **Scheduling**: Set time-based rules for automatic lighting changes

## Post-conditions

- **Success Condition**: Light reflects requested state; app displays current status
- **Failure Condition**: App displays error with retry option; light maintains previous state

## Tier Feature Matrix

| Capability | Starter | Home | Premium |
|------------|---------|------|---------|
| On/Off, brightness, color | ✓ | ✓ | ✓ |
| Light groups | — | ✓ | ✓ |
| Scheduled scenes | — | ✓ | ✓ |
| Voice control | — | ✓ | ✓ |
| AI adaptive lighting | — | — | ✓ |
| Circadian rhythm automation | — | — | ✓ |
