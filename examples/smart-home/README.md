# Smart Home Control System - Example

This folder contains a minimal example of a requirements hierarchy following
the sara templates.

## Structure

```
smart-home/
├── solutions/
│   └── SOL-SMARTHOME.md              # SOL-001: Top-level solution
├── use_cases/
│   ├── UC-LIGHTS.md                  # UC-001: Lighting control use case
│   └── UC-SECURITY.md                # UC-002: Security monitoring use case
├── scenarios/
│   ├── SCEN-DIMMER.md                # SCEN-001: Dimmer adjustment scenario
│   └── SCEN-INTRUSION.md             # SCEN-002: Intrusion detection scenario
├── system_requirements/
│   ├── SYSREQ-LATENCY.md             # SYSREQ-001: Command latency requirement
│   └── SYSREQ-ALERT.md               # SYSREQ-002: Alert delivery requirement
├── system_architecture/
│   └── SYSARCH-COMM.md               # SYSARCH-001: Communication architecture
├── software_requirements/
│   ├── SWREQ-MQTTCLIENT.md           # SWREQ-001: MQTT client library
│   └── SWREQ-PUSHSDK.md              # SWREQ-002: Push SDK integration
├── hardware_requirements/
│   ├── HWREQ-ZIGBEE.md               # HWREQ-001: Zigbee radio module
│   └── HWREQ-HUB.md                  # HWREQ-002: Central hub hardware
├── detailed_design/
│   ├── SWDD-MQTT.md                  # SWDD-001: MQTT protocol design (SW)
│   ├── SWDD-ALERTPUSH.md             # SWDD-002: Push notification design (SW)
│   ├── HWDD-ZIGBEE.md                # HWDD-001: Zigbee module design (HW)
│   └── HWDD-HUBBOARD.md              # HWDD-002: Hub board design (HW)
└── adrs/
    ├── ADR-HYBRIDHUB.md              # ADR-001: Hub-based hybrid architecture
    ├── ADR-MQTT.md                   # ADR-002: Use MQTT for device communication
    └── ADR-ZIGBEE.md                 # ADR-003: Use Zigbee for wireless mesh
```

## Traceability Graph

```
SOL-001
├── UC-001
│   └── SCEN-001
│       └── SYSREQ-001 ──┐
└── UC-002               │
    └── SCEN-002         │
        └── SYSREQ-002 ──┤
                         ▼
                  SYSARCH-001 ◄── ADR-001
                 /           \
    ┌───────────┘             └───────────┐
    ▼                                     ▼
SWREQ-001                             HWREQ-001
SWREQ-002                             HWREQ-002
    │                                     │
    ▼                                     ▼
SWDD-001 ◄── ADR-002                  HWDD-001 ◄── ADR-003
SWDD-002                              HWDD-002
```

## Relationships

| Child       | Relationship | Parent                 |
|-------------|--------------|------------------------|
| UC-001      | refines      | SOL-001                |
| UC-002      | refines      | SOL-001                |
| SCEN-001    | refines      | UC-001                 |
| SCEN-002    | refines      | UC-002                 |
| SYSREQ-001  | derives_from | SCEN-001               |
| SYSREQ-002  | derives_from | SCEN-002               |
| SYSARCH-001 | satisfies    | SYSREQ-001, SYSREQ-002 |
| SWREQ-001   | derives_from | SYSARCH-001            |
| SWREQ-002   | derives_from | SYSARCH-001            |
| HWREQ-001   | derives_from | SYSARCH-001            |
| HWREQ-002   | derives_from | SYSARCH-001            |
| SWDD-001    | satisfies    | SWREQ-001              |
| SWDD-002    | satisfies    | SWREQ-002              |
| HWDD-001    | satisfies    | HWREQ-001              |
| HWDD-002    | satisfies    | HWREQ-002              |
| ADR-001     | justifies    | SYSARCH-001            |
| ADR-002     | justifies    | SWDD-001               |
| ADR-003     | justifies    | HWDD-001               |

## Element Types

| Type                          | Prefix   | Description                          |
|-------------------------------|----------|--------------------------------------|
| solution                      | SOL-     | Top-level product/system definition  |
| use_case                      | UC-      | User-facing functionality            |
| scenario                      | SCEN-    | Specific flow within a use case      |
| system_requirement            | SYSREQ-  | System-level SHALL statements        |
| system_architecture           | SYSARCH- | High-level technical architecture    |
| software_requirement          | SWREQ-   | Software-specific requirements       |
| hardware_requirement          | HWREQ-   | Hardware-specific requirements       |
| software_detailed_design      | SWDD-    | Software design documents            |
| hardware_detailed_design      | HWDD-    | Hardware design documents            |
| architecture_decision_record  | ADR-     | Cross-cutting design decisions       |
