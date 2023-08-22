import logging

from openff.qcsubmit.results import TorsionDriveResultCollection
from qcportal import FractalClient

client = FractalClient()

collection = TorsionDriveResultCollection.parse_file(
    "../testfiles/core-td.json"
)

logging.getLogger("openff.toolkit").setLevel(logging.ERROR)
records_and_molecules = collection.to_records()
