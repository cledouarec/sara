---
id: "SCEN-INTRUSION"
type: scenario
name: "Door Sensor Triggered While Away"
description: >
  System detects door opening while homeowner is away and armed
refines:
  - "UC-SECURITY"
---

# Scenario: Door Sensor Triggered While Away

## Overview

Validates the intrusion detection flow from sensor trigger through escalation
to alarm and professional monitoring notification when no disarm code is entered.

## Initial State

- Security system is armed in "Away" mode
- Front door sensor is active and online
- Homeowner is not present (mobile app shows location: Away)
- Entry countdown is configured for 30 seconds

## Trigger

Front door contact sensor detects door opening (magnetic reed switch breaks).

## Step-by-Step Flow

1. **Reaction**: System logs sensor trigger event with timestamp
2. **Reaction**: System sends push notification to homeowner: "Front door opened - Away mode"
3. **Reaction**: System activates entry camera and begins recording
4. **Reaction**: System starts 30-second countdown, displays on entry keypad
5. **Action**: [No user action - countdown expires without valid code]
6. **Reaction**: System triggers alarm siren (interior and exterior)
7. **Reaction**: System sends alert to security monitoring service with event details
8. **Reaction**: App updates to show "Alarm Active" with camera feed

## Expected Outcome

- **Success Condition**: Alarm triggered; monitoring service notified; event logged with video
- **Verification**: Event log contains DOOR_OPEN → COUNTDOWN_START → ALARM_TRIGGERED sequence

## Exceptions & Edge Cases

- **Valid Code Entered**: If user enters correct code within 30s, countdown cancels and system disarms
- **Camera Offline**: Alarm proceeds without recording; system logs camera failure
- **Monitoring Service Unreachable**: System retries 3 times, then sends SMS fallback to homeowner
