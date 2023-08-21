import logging

from openff.qcsubmit.results import OptimizationResultCollection
from qcportal import FractalClient

client = FractalClient()

collection = OptimizationResultCollection.parse_file(
    "../testfiles/core-opt.json"
)

logging.getLogger("openff.toolkit").setLevel(logging.ERROR)
records_and_molecules = collection.to_records()
