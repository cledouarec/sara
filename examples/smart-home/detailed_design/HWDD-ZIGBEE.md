---
id: "HWDD-ZIGBEE"
type: hardware_detailed_design
name: "Zigbee Module Integration"
description: >
  Detailed design for Zigbee radio module PCB integration
satisfies:
  - "HWREQ-ZIGBEE"
---

# Zigbee Module Integration

## Component Selection

Selected module: Silicon Labs EFR32MG21

## PCB Layout

- Module placement: Edge of PCB for optimal antenna performance
- Keep-out zone: 10mm around antenna area
- Ground plane: Solid ground under module except antenna feed

## Schematic

```mermaid
flowchart LR
    subgraph power["Power Supply"]
        vcc[VCC 3.3V]
        c1[C1 100nF]
        gnd[GND]
    end

    subgraph module["EFR32MG21"]
        radio[Zigbee Radio]
        gpio[GPIO/SPI]
    end

    mcu[Host MCU]
    antenna([PCB Antenna])

    vcc --> c1
    c1 --> radio
    radio --> gnd
    gpio <--> mcu
    radio --> antenna
```

## RF Performance

- Conducted output power: +10 dBm
- Receiver sensitivity: -104 dBm
- Antenna gain: +2 dBi (PCB trace antenna)

## PCB Layout Zones

```mermaid
flowchart LR
    subgraph pcb["PCB Layout"]
        mcu[Host MCU]
        spi[SPI Bus]

        subgraph rf["RF Section"]
            zigbee[EFR32MG21]
            keepout[Keep-out 10mm]
            antenna([PCB Antenna])
        end
    end

    gnd[(Solid Ground Plane)]

    mcu --- spi
    spi --- zigbee
    zigbee --- keepout
    keepout --- antenna
    zigbee -.-> gnd
```
