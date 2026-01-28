# Smart Home Control System - Example

This folder contains a minimal example of a requirements hierarchy following
the sara templates.

## Structure

```
smart-home/
├── solutions/
│   └── SOL-SMARTHOME.md              # Top-level solution
├── use_cases/
│   ├── UC-LIGHTS.md                  # Lighting control use case
│   └── UC-SECURITY.md                # Security monitoring use case
├── scenarios/
│   ├── SCEN-DIMMER.md                # Dimmer adjustment scenario
│   └── SCEN-INTRUSION.md             # Intrusion detection scenario
├── system_requirements/
│   ├── SYSREQ-LATENCY.md             # Command latency requirement
│   └── SYSREQ-ALERT.md               # Alert delivery requirement
├── system_architecture/
│   └── SYSARCH-COMM.md               # Communication architecture
├── software_requirements/
│   ├── SWREQ-MQTTCLIENT.md           # MQTT client library
│   └── SWREQ-PUSHSDK.md              # Push SDK integration
├── hardware_requirements/
│   ├── HWREQ-ZIGBEE.md               # Zigbee radio module
│   └── HWREQ-HUB.md                  # Central hub hardware
├── detailed_design/
│   ├── SWDD-MQTT.md                  # MQTT protocol design (SW)
│   ├── SWDD-ALERTPUSH.md             # Push notification design (SW)
│   ├── HWDD-ZIGBEE.md                # Zigbee module design (HW)
│   └── HWDD-HUBBOARD.md              # Hub board design (HW)
└── adrs/
    ├── ADR-001.md                    # Hub-based hybrid architecture
    ├── ADR-002.md                    # Use MQTT for device communication
    └── ADR-003.md                    # Use Zigbee for wireless mesh
```

## Traceability Graph

```
SOL-SMARTHOME
├── UC-LIGHTS
│   └── SCEN-DIMMER
│       └── SYSREQ-LATENCY ─────┐
└── UC-SECURITY                 │
    └── SCEN-INTRUSION          │
        └── SYSREQ-ALERT ───────┤
                                ▼
                         SYSARCH-COMM ◄── ADR-001
                        /            \
           ┌───────────┘              └───────────┐
           ▼                                      ▼
    SWREQ-MQTTCLIENT                       HWREQ-ZIGBEE
    SWREQ-PUSHSDK                          HWREQ-HUB
           │                                      │
           ▼                                      ▼
    SWDD-MQTT ◄── ADR-002               HWDD-ZIGBEE ◄── ADR-003
    SWDD-ALERTPUSH                        HWDD-HUBBOARD
```

## Relationships

| Child            | Relationship | Parent           |
|------------------|--------------|------------------|
| UC-LIGHTS        | refines      | SOL-SMARTHOME    |
| UC-SECURITY      | refines      | SOL-SMARTHOME    |
| SCEN-DIMMER      | refines      | UC-LIGHTS        |
| SCEN-INTRUSION   | refines      | UC-SECURITY      |
| SYSREQ-LATENCY   | derives_from | SCEN-DIMMER      |
| SYSREQ-ALERT     | derives_from | SCEN-INTRUSION   |
| SYSARCH-COMM     | satisfies    | SYSREQ-LATENCY, SYSREQ-ALERT |
| SWREQ-MQTTCLIENT | derives_from | SYSARCH-COMM     |
| SWREQ-PUSHSDK    | derives_from | SYSARCH-COMM     |
| HWREQ-ZIGBEE     | derives_from | SYSARCH-COMM     |
| HWREQ-HUB        | derives_from | SYSARCH-COMM     |
| SWDD-MQTT        | satisfies    | SWREQ-MQTTCLIENT |
| SWDD-ALERTPUSH   | satisfies    | SWREQ-PUSHSDK    |
| HWDD-ZIGBEE      | satisfies    | HWREQ-ZIGBEE     |
| HWDD-HUBBOARD    | satisfies    | HWREQ-HUB        |
| ADR-001          | justifies    | SYSARCH-COMM |
| ADR-002          | justifies    | SWDD-MQTT |
| ADR-003          | justifies    | HWDD-ZIGBEE |

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
