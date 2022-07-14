import { IDerivation, Document, Register, ByHourSource } from 'flow/ops/TENANT/stats-by-hour';

// Implementation for derivation ops-catalog/template.flow.yaml#/collections/ops~1TENANT~1stats-by-hour/derivation.
export class Derivation implements IDerivation {
    byHourPublish(source: ByHourSource, _register: Register, _previous: Register): Document[] {
        const ts = new Date(source.ts);
        ts.setUTCMilliseconds(0);
        ts.setUTCSeconds(0);
        ts.setUTCMinutes(0);
        source.ts = ts.toISOString();
        return [source];
    }
}
