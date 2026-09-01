# IoT, Native

> **Design intent.** This page explains target behaviour; the README's
> status table says what is built today. Specification, not claim.

IoT is not an integration; it is a declared property of every module.

## Per-module telemetry

Every module's manifest declares the telemetry that is *meaningful* to its
domain (a full table in modules.md):

| Module | Telemetry |
|---|---|
| pln | POS & shelf sensors - weather feeds |
| src | inbound GPS - gate scans - cold-chain probes |
| trf | line PLCs - OEE counters - vision QA |
| ord | e-comm & POS event streams - chat ops |
| crm | connected-product telemetry - app events |
| ful | fleet GPS - geo-fences - cold-chain probes |
| ret | smart RMA kiosks - return-drop scans |
| inv | RFID - smart shelves - drone cycle counts |
| srm | supplier port telemetry - cert feeds |
| ctr | usage meters - SLA probes - e-sign pads |
| fin | POS terminals - metered-usage events |
| tsk | shop-floor badges - wearable pings |
| prj | site sensors - asset & crew trackers |
| net | edge agent fleet - device shadows |

## Pipeline

1. **Edge gateways** buffer with store-and-forward (a warehouse with a
   dead uplink for six hours loses nothing).
2. **`kernel.ingest`** accepts MQTT/edge streams alongside CSV / XLSX / API;
   every arrival is a master-log entry; unparsable payloads go to the
   dead-letter queue — never silently dropped.
3. **Device registry + shadows**: devices are master data with bitemporal
   columns; shadows hold last-known state per device.
4. **Module craft consumes it**: OEE counters feed TRANSFORM's SPC;
   RFID counts feed INVENTORY's cycle counts; cold-chain probes feed
   SOURCE/FULFILL quality holds and FINANCE landed cost.

## Rules

- Telemetry is data like any other: permission-checked, ledger-stamped,
  tenant-chained.
- Devices authenticate individually; a compromised gateway cannot spoof
  another tenant's fleet (salted, per-tenant hashing again).
