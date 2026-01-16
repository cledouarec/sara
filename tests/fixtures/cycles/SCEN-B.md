---
id: "SCEN-B"
type: scenario
name: "Scenario B"
description: "Part of circular reference: B -> A -> B"
refines:
  - "SCEN-A"
---

# Scenario: Scenario B

This scenario refines SCEN-A, which in turn refines SCEN-B, creating a cycle.

## Purpose

Test fixture for circular reference detection validation.
