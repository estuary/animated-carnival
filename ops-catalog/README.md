# Control-plane Ops Catalog Template

This is the Flow catalog template that is used to provision new tenants within the control plane.

Whenever a new tenant is provisioned, the `TENANT` placeholder string within `template-bundled.flow.json`
is replaced with the actual tenant, and the specifications of the catalog are then applied (excepting tests).

The contents of `template-bundled.flow.json` is compiled into the control-plane agent.
To re-generate this file, run:

```bash
flowctl raw bundle --source ops-catalog/template.flow.yaml > ops-catalog/template-bundled.flow.json
```

The template expects that a new partition of table `task_stats`, defined in SQL
migration `10_stats.sql`, is explicitly created by the agent prior to the materialization
being applied.