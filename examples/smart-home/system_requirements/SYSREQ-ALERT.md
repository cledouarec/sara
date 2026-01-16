---
id: "SYSREQ-ALERT"
type: system_requirement
name: "Security Alert Delivery"
description: >
  Timely delivery of security alerts to users
specification: >
  The system SHALL deliver security alerts to user devices within 5 seconds
  of event detection
derives_from:
  - "SCEN-INTRUSION"
---

# Security Alert Delivery

Critical security events must be communicated to users immediately.

## Rationale

Security scenarios require rapid notification to allow homeowners to respond
appropriately. Delays in alert delivery could compromise home security.

## Verification

- Trigger test security event
- Measure time from sensor activation to push notification receipt
- Must complete within 5 seconds under normal network conditions
