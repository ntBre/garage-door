# convert the output of garage-door back to actual records and molecules as if
# you called to_records directly

import json
import logging

import numpy as np
from openff.toolkit import Molecule
from openff.units import unit
from qcportal.models import TorsionDriveRecord

logging.getLogger("openff.toolkit").setLevel(logging.ERROR)

with open("/tmp/td.json", "r") as infile:
    results = json.load(infile)

for r in results:
    [record, cmiles, conformers] = r
    molecule = Molecule.from_mapped_smiles(cmiles, allow_undefined_stereo=True)
    molecule.add_conformer(
        np.array(conformers[0], float).reshape(-1, 3) * unit.bohr
    )
    record = TorsionDriveRecord.parse_obj(record)
    print(record, molecule)
