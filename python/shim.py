# convert the output of garage-door back to actual records and molecules as if
# you called to_records directly

import json
import logging
import sys

import numpy as np
import qcelemental
from openff.toolkit import Molecule
from openff.units import unit
from qcportal.models import OptimizationRecord, TorsionDriveRecord
from tqdm import tqdm

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
for r in tqdm(
    results, desc="Converting to records and molecules", total=len(results)
):
    [record, cmiles, conformers] = r
    molecule = Molecule.from_mapped_smiles(cmiles, allow_undefined_stereo=True)
    molecule._conformers = [
        np.array(conformers[0], float).reshape(-1, 3)
        * qcelemental.constants.bohr2angstroms
        * unit.angstrom
    ]
    record = typ.parse_obj(record)
    # print(record, molecule)

# most of the time is in calling `from_mapped_smiles`: 363 / 424 seconds.
# there's nothing I can really do about that except try to multiprocess it. but
# where are those extra 60 seconds from? 17 seconds from record.__init__ as
# measured by commenting out molecule stuff. 15 seconds from add_conformer, as
# measured by adding the same conformer to the same molecule over and over. if
# you also do the np.array stuff each time, this goes up to 33 seconds. If you
# use the known shape instead of -1, it doesn't matter
