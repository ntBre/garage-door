import logging

from openff.qcsubmit.results import OptimizationResultCollection

data = "../../projects/benchmarking/datasets/filtered-industry.json"
collection = OptimizationResultCollection.parse_file(data)

logging.getLogger("openff.toolkit").setLevel(logging.ERROR)

records_and_molecules = collection.to_records()
