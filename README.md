# garage-door
like a portal but a lot bigger

# Usage

`garage-door` is a tool for quickly retrieving data sets from QCFractal servers
and converting the results to a sequence of QCPortal [Record]s and OpenFF
[Molecule]s like a call to [openff.toolkit.BasicResultCollection.to_records].
Well, it almost does that, except for the fact that constructing a `Molecule`
outside of the Python code is difficult. So it actually prints `Record`s and the
building blocks for the corresponding `Molecule`s in JSON format for further
processing by Python. See [shim.py](python/shim.py) for an example of that.

There are two subcommands for `garage-door`: `get` and `convert`. The first of
these fetches a named dataset from QCFractal:

``` shell
garage-door get "OpenFF multiplicity correction torsion drive data v1.1" \
	    --dataset-type TorsionDrive
```

Unfortunately, I haven't figured out a good way to detect the type of the
dataset, so the `--dataset-type` flag is required for now.

The `convert` subcommand instead reads an existing dataset file with contents
like

``` json
{
  "entries": {
    "https://api.qcarchive.molssi.org:443/": [
      {
        "type": "torsion",
        "record_id": "107276605",
        "cmiles": "[H:12][c:1]1[c:2]([c:3]([c:4]([c:5]([c:6]1[H:16])[H:15])[S:7](=[O:8])(=[O:9])[N:10]([H:17])[N:11]([H:18])[H:19])[H:14])[H:13]",
        "inchi_key": "VJRITMATACIYAF-UHFFFAOYNA-N"
      }
    ]
  }
}
```

and then runs the rest of the code.

[Record]: https://github.com/MolSSI/QCPortal/blob/ff6f8bdf733b648e927223c89126a3ba37f88b69/qcportal/models/records.py#L251
[Molecule]: https://docs.openforcefield.org/projects/toolkit/en/latest/api/generated/openff.toolkit.topology.Molecule.html#openff.toolkit.topology.Molecule
[openff.toolkit.BasicResultCollection.to_records]: https://docs.openforcefield.org/projects/qcsubmit/en/latest/api/generated/openff.qcsubmit.results.BasicResultCollection.html?highlight=to_records#openff.qcsubmit.results.BasicResultCollection.to_records
