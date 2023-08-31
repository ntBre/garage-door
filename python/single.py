from qcportal import FractalClient
from openff.qcsubmit.results import BasicResultCollection

ds_cache = "single.json"
td_datasets = ["OpenFF BCC Refit Study COH v1.0"]

client = FractalClient()
dataset = BasicResultCollection.from_server(
    client=client,
    datasets=td_datasets,
    spec_name="resp-2-vacuum",
)
with open(ds_cache, "w") as out:
    out.write(dataset.json(indent=2))
