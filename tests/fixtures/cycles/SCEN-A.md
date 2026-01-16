---
id: "SCEN-A"
type: scenario
name: "Scenario A"
description: "Part of circular reference: A -> B -> A"
refines:
  - "SCEN-B"
---

# Scenario: Scenario A

This scenario refines SCEN-B, which in turn refines SCEN-A, creating a cycle.

## Purpose

Test fixture for circular reference detection validation.
