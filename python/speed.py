import logging

from openff.qcsubmit.results import TorsionDriveResultCollection
from qcportal import FractalClient

client = FractalClient()

print("retrieving collection")
collection = TorsionDriveResultCollection.from_server(
    client=client,
    datasets=[
        "OpenFF multiplicity correction torsion drive data v1.1",
    ],
    spec_name="default",
)

logging.getLogger("openff.toolkit").setLevel(logging.ERROR)
print("calling to_records")
records_and_molecules = collection.to_records()

# for record, molecule in records_and_molecules:
#     print(record)
#     print(molecule)
