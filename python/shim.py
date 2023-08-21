# convert the output of garage-door back to actual records and molecules as if
# you called to_records directly

import json
import logging
import sys

import numpy as np
from openff.toolkit import Molecule
from openff.units import unit
from qcportal.models import OptimizationRecord, TorsionDriveRecord

logging.getLogger("openff.toolkit").setLevel(logging.ERROR)

arg = sys.argv[1]

with open(arg, "r") as infile:
    results = json.load(infile)


def get_type(results):
    typ = results[0][0]["procedure"]
    match typ:
        case "optimization":
            return OptimizationRecord
        case "torsiondrive":
            return TorsionDriveRecord
    raise ValueError(f"Unrecognized record type: {typ}")


typ = get_type(results)
for r in results:
    [record, cmiles, conformers] = r
    molecule = Molecule.from_mapped_smiles(cmiles, allow_undefined_stereo=True)
    molecule.add_conformer(
        np.array(conformers[0], float).reshape(-1, 3) * unit.bohr
    )
    record = typ.parse_obj(record)
    print(record, molecule)
