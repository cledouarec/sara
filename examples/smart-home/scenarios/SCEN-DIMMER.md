---
id: "SCEN-DIMMER"
type: scenario
name: "Adjust Living Room Dimmer"
description: >
  User adjusts brightness of living room lights using the mobile app
refines:
  - "UC-LIGHTS"
---

# Scenario: Adjust Living Room Dimmer

## Overview

Demonstrates the brightness adjustment flow for a light group, validating
that commands propagate to multiple devices simultaneously with visual feedback.

## Initial State

- Living room light group contains 3 dimmable bulbs
- All bulbs are currently at 100% brightness
- User has Homeowner permissions
- Hub is online and connected

## Trigger

User opens the lighting control panel and selects the "Living Room" group.

## Step-by-Step Flow

1. **Action**: User navigates to Lighting → Living Room group
2. **Reaction**: System displays group with current brightness (100%) and individual bulb status
3. **Action**: User drags brightness slider to 30%
4. **Reaction**: System sends dim command to all 3 bulbs via Zigbee
5. **Reaction**: Bulbs transition to 30% brightness within 2 seconds
6. **Reaction**: App updates slider position and displays "30%" confirmation

## Expected Outcome

- **Success Condition**: All 3 bulbs display 30% brightness; app shows synchronized state
- **Verification**: Query device states via API returns brightness=30 for each bulb

## Exceptions & Edge Cases

- **Bulb Offline**: If one bulb is unreachable, system dims available bulbs and displays partial success warning
- **Network Latency**: If response exceeds 2 seconds, app shows "Applying..." indicator
- **Permission Denied**: Guest users without group control see read-only slider
